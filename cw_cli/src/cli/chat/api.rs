use std::env;

use amzn_codewhisperer_streaming_client::types::{
    ChatMessage,
    ChatResponseStream,
    ChatTriggerType,
    ConversationState,
    EnvState,
    EnvironmentVariable,
    GitState,
    ShellHistoryEntry,
    ShellState,
    UserInputMessage,
    UserInputMessageContext,
};
use amzn_codewhisperer_streaming_client::Client;
use eyre::Result;
use fig_settings::history::OrderBy;
use fig_util::Shell;
use once_cell::sync::Lazy;
use regex::Regex;
use tokio::sync::mpsc::{
    UnboundedReceiver,
    UnboundedSender,
};
use tracing::error;

use super::ApiResponse;

// Max constants for length of strings and lists, use these to truncate elements
// to ensure the API request is valid

// https://code.amazon.com/packages/AWSVectorConsolasPlatformModel/blobs/heads/mainline/--/model/types/env_types.smithy
const MAX_ENV_VAR_LIST_LEN: usize = 100;
const MAX_ENV_VAR_KEY_LEN: usize = 256;
const MAX_ENV_VAR_VALUE_LEN: usize = 256;
const MAX_CURRENT_WORKING_DIRECTORY_LEN: usize = 256;

// https://code.amazon.com/packages/AWSVectorConsolasPlatformModel/blobs/mainline/--/model/types/git_types.smithy
const MAX_GIT_STATUS_LEN: usize = 4096;

// https://code.amazon.com/packages/AWSVectorConsolasPlatformModel/blobs/mainline/--/model/types/shell_types.smithy
const MAX_SHELL_HISTORY_LIST_LEN: usize = 20;
const MAX_SHELL_HISTORY_COMMAND_LEN: usize = 1024;
const MAX_SHELL_HISTORY_DIRECTORY_LEN: usize = 256;

/// Regex for the context modifiers `@git`, `@env`, and `@history`
static CONTEXT_MODIFIER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"@(git|env|history) ?").unwrap());

fn truncate_safe(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }

    let mut byte_count = 0;
    let mut char_indices = s.char_indices();

    for (byte_idx, _) in &mut char_indices {
        if byte_count + (byte_idx - byte_count) > max_bytes {
            break;
        }
        byte_count = byte_idx;
    }

    &s[..byte_count]
}

/// The context modifiers that are used in a specific chat message
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ContextModifiers {
    env: bool,
    history: bool,
    git: bool,
}

/// Convert the `input` into the [ContextModifiers] and a string with them removed
fn input_to_modifiers(input: String) -> (ContextModifiers, String) {
    let mut modifiers = ContextModifiers::default();

    for capture in CONTEXT_MODIFIER_REGEX.captures_iter(&input) {
        let modifier = capture.get(1).expect("regex has a captrue group").as_str();

        match modifier {
            "git" => modifiers.git = true,
            "env" => modifiers.env = true,
            "history" => modifiers.history = true,
            _ => unreachable!(),
        }
    }

    (modifiers, input)
}

fn build_shell_history() -> Option<Vec<ShellHistoryEntry>> {
    let mut shell_history = vec![];

    if let Ok(commands) = fig_settings::history::History::new().rows(
        None,
        vec![OrderBy::new(
            fig_settings::history::HistoryColumn::Id,
            fig_settings::history::Order::Desc,
        )],
        MAX_SHELL_HISTORY_LIST_LEN,
        0,
    ) {
        for command in commands.into_iter().filter(|c| c.command.is_some()).rev() {
            shell_history.push(
                ShellHistoryEntry::builder()
                    .command(truncate_safe(
                        &command.command.expect("command is filtered on"),
                        MAX_SHELL_HISTORY_COMMAND_LEN,
                    ))
                    .set_directory(
                        command
                            .cwd
                            .map(|cwd| truncate_safe(&cwd, MAX_SHELL_HISTORY_DIRECTORY_LEN).into()),
                    )
                    .set_exit_code(command.exit_code)
                    .build()
                    .expect("command is provided"),
            );
        }
    }

    if shell_history.is_empty() {
        None
    } else {
        Some(shell_history)
    }
}

fn build_shell_state(shell_history: bool) -> ShellState {
    let mut shell_state_builder = ShellState::builder();

    // Try to grab the shell from the parent process via the `Shell::current_shell`,
    // then try the `SHELL` env, finally just report bash
    let shell = Shell::current_shell()
        .or_else(|| {
            let shell_name = env::var("SHELL").ok()?;
            Shell::try_find_shell(&shell_name)
        })
        .unwrap_or(Shell::Bash);

    shell_state_builder = shell_state_builder.shell_name(shell.to_string());

    if shell_history {
        shell_state_builder = shell_state_builder.set_shell_history(build_shell_history());
    }

    shell_state_builder.build().expect("shell name is provided")
}

fn build_env_state(environment_variables: bool) -> EnvState {
    let mut env_state_builder = EnvState::builder()
        .current_working_directory(truncate_safe(
            &env::current_dir().unwrap_or_default().to_string_lossy(),
            MAX_CURRENT_WORKING_DIRECTORY_LEN,
        ))
        .operating_system(env::consts::OS);

    if environment_variables {
        for (key, value) in env::vars().take(MAX_ENV_VAR_LIST_LEN) {
            env_state_builder = env_state_builder.environment_variables(
                EnvironmentVariable::builder()
                    .key(truncate_safe(&key, MAX_ENV_VAR_KEY_LEN))
                    .value(truncate_safe(&value, MAX_ENV_VAR_VALUE_LEN))
                    .build(),
            );
        }
    }

    env_state_builder.build()
}

async fn build_git_state() -> Option<GitState> {
    // git status --porcelain=v1 -b
    let output = tokio::process::Command::new("git")
        .args(["status", "--porcelain=v1", "-b"])
        .output()
        .await
        .ok()?;

    Some(
        GitState::builder()
            .status(truncate_safe(
                &String::from_utf8_lossy(&output.stdout),
                MAX_GIT_STATUS_LEN,
            ))
            .build(),
    )
}

async fn try_send_message(
    client: Client,
    tx: &UnboundedSender<ApiResponse>,
    conversation_state: ConversationState,
) -> Result<()> {
    let mut res = client
        .generate_assistant_response()
        .conversation_state(conversation_state)
        .send()
        .await?;

    loop {
        match res.generate_assistant_response_response.recv().await {
            Ok(Some(stream)) => match stream {
                ChatResponseStream::AssistantResponseEvent(response) => {
                    tx.send(ApiResponse::Text(response.content))?;
                },
                ChatResponseStream::MessageMetadataEvent(_event) => {},
                ChatResponseStream::FollowupPromptEvent(_event) => {},
                ChatResponseStream::CodeReferenceEvent(_event) => {},
                ChatResponseStream::SupplementaryWebLinksEvent(_event) => {},
                ChatResponseStream::InvalidStateEvent(_event) => {},
                _ => {},
            },
            Ok(None) => break,
            Err(err) => return Err(err.into()),
        }
    }

    Ok(())
}

pub(super) async fn send_message(client: Client, input: String) -> Result<UnboundedReceiver<ApiResponse>> {
    let (ctx, input) = input_to_modifiers(input);

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    let mut context_builder = UserInputMessageContext::builder()
        .shell_state(build_shell_state(ctx.history))
        .env_state(build_env_state(ctx.env));

    if ctx.git {
        if let Some(git_state) = build_git_state().await {
            context_builder = context_builder.git_state(git_state);
        }
    }

    let user_input_message = UserInputMessage::builder()
        .content(input)
        .user_input_message_context(context_builder.build())
        .build()?;

    let conversation_state = ConversationState::builder()
        .current_message(ChatMessage::UserInputMessage(user_input_message))
        .chat_trigger_type(ChatTriggerType::Manual)
        .build()?;

    tokio::spawn(async move {
        if let Err(err) = try_send_message(client, &tx, conversation_state).await {
            error!(%err);
            tx.send(ApiResponse::Error).ok();
            return;
        }

        // Try to end stream
        tx.send(ApiResponse::End).ok();
    });

    Ok(rx)
}

#[cfg(test)]
mod tests {
    use fig_api_client::ai::{
        cw_endpoint,
        cw_streaming_client,
    };
    use tokio::io::AsyncWriteExt;

    use super::*;

    #[test]
    fn test_truncate_safe() {
        assert_eq!(truncate_safe("Hello World", 5), "Hello");
        assert_eq!(truncate_safe("Hello ", 5), "Hello");
        assert_eq!(truncate_safe("Hello World", 11), "Hello World");
        assert_eq!(truncate_safe("Hello World", 15), "Hello World");
    }

    #[tokio::test]
    #[ignore = "not in ci"]
    async fn test_send_message() {
        let client = cw_streaming_client(cw_endpoint()).await;
        let question = "@git Explain my git status.".to_string();

        let mut rx = send_message(client.clone(), question).await.unwrap();

        while let Some(res) = rx.recv().await {
            match res {
                ApiResponse::Text(text) => {
                    let mut stderr = tokio::io::stderr();
                    stderr.write_all(text.as_bytes()).await.unwrap();
                    stderr.flush().await.unwrap();
                },
                ApiResponse::End => break,
                ApiResponse::Error => panic!(),
            }
        }
    }

    #[test]
    fn test_input_to_modifiers() {
        let (modifiers, input) = input_to_modifiers("How do I use git?".to_string());
        assert_eq!(modifiers, ContextModifiers::default());
        assert_eq!(input, "How do I use git?");

        let (modifiers, input) = input_to_modifiers("@git @env @history How do I use git?".to_string());
        assert_eq!(modifiers, ContextModifiers {
            env: true,
            history: true,
            git: true
        });
        assert_eq!(input, "How do I use git?");

        let (modifiers, input) = input_to_modifiers("@git How do I use git?".to_string());
        assert_eq!(modifiers, ContextModifiers {
            env: false,
            history: false,
            git: true
        });
        assert_eq!(input, "How do I use git?");

        let (modifiers, input) = input_to_modifiers("@env How do I use git?".to_string());
        assert_eq!(modifiers, ContextModifiers {
            env: true,
            history: false,
            git: false
        });
        assert_eq!(input, "How do I use git?");
    }

    #[test]
    fn test_shell_state() {
        let shell_state = build_shell_state(true);

        for history in shell_state.shell_history() {
            println!(
                "{} {:?} {:?}",
                history.command(),
                history.directory(),
                history.exit_code()
            );
        }
    }

    #[test]
    fn test_env_state() {
        let env_state = build_env_state(true);
        assert!(!env_state.environment_variables().is_empty());
        assert!(!env_state.current_working_directory().unwrap().is_empty());
        assert!(!env_state.operating_system().unwrap().is_empty());
        println!("{env_state:?}");
    }

    #[tokio::test]
    async fn test_git_state() {
        let git_state = build_git_state().await.unwrap();
        println!("{git_state:?}");
        println!("status: {:?}", git_state.status.unwrap());
    }
}
