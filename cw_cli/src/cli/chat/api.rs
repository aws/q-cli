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
use fig_util::Shell;
use once_cell::sync::Lazy;
use regex::Regex;
use tokio::sync::mpsc::{
    Receiver,
    Sender,
};
use tracing::error;

use super::ApiResponse;

/// Regex for the context modifiers `@git`, `@env`, and `@history`
static CONTEXT_MODIFIER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"@(git|env|history) ?").unwrap());

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

    let input = CONTEXT_MODIFIER_REGEX.replace_all(&input, "").to_string();

    (modifiers, input)
}

fn build_shell_history() -> Option<Vec<ShellHistoryEntry>> {
    let mut shell_history = vec![];

    if let Ok(commands) = fig_settings::history::History::new().rows(None, vec![], 10, 0) {
        for command in commands.into_iter().filter(|c| c.command.is_some()) {
            shell_history.push(
                ShellHistoryEntry::builder()
                    .command(command.command.expect("command is filtered on"))
                    .set_directory(command.cwd)
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
        .current_working_directory(env::current_dir().unwrap_or_default().to_string_lossy())
        .operating_system(env::consts::OS);

    if environment_variables {
        for (key, value) in env::vars() {
            env_state_builder =
                env_state_builder.environment_variables(EnvironmentVariable::builder().key(key).value(value).build());
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
            .status(String::from_utf8_lossy(&output.stdout))
            .build(),
    )
}

async fn try_send_message(
    client: Client,
    tx: &Sender<ApiResponse>,
    conversation_state: ConversationState,
) -> Result<()> {
    let mut res = client
        .generate_assistant_response()
        .conversation_state(conversation_state)
        .send()
        .await?;

    while let Ok(Some(stream)) = res.generate_assistant_response_response.recv().await {
        match stream {
            ChatResponseStream::AssistantResponseEvent(response) => {
                tx.send(ApiResponse::Text(response.content)).await?;
            },
            ChatResponseStream::MessageMetadataEvent(_event) => {},
            ChatResponseStream::FollowupPromptEvent(_event) => {},
            ChatResponseStream::CodeReferenceEvent(_event) => {},
            ChatResponseStream::SupplementaryWebLinksEvent(_event) => {},
            ChatResponseStream::InvalidStateEvent(_event) => {},
            _ => {},
        }
    }

    Ok(())
}

pub(super) async fn send_message(client: Client, input: String) -> Result<Receiver<ApiResponse>> {
    let (ctx, input) = input_to_modifiers(input);

    let (tx, rx) = tokio::sync::mpsc::channel(8);

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
        }

        // Try to end stream
        tx.send(ApiResponse::End).await.ok();
    });

    Ok(rx)
}

#[cfg(test)]
mod tests {
    use fig_api_client::ai::{
        cw_streaming_client,
        Endpoint,
    };
    use tokio::io::AsyncWriteExt;

    use super::*;

    #[tokio::test]
    #[ignore = "not in ci"]
    async fn test_send_message() {
        let client = cw_streaming_client(Endpoint::Alpha).await;
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
        println!("{shell_state:?}");
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
