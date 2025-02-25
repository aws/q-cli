use std::collections::VecDeque;
use std::env;
use std::path::Path;
use std::sync::LazyLock;

use eyre::{
    Result,
    bail,
};
use fig_api_client::model::{
    ChatMessage,
    ConversationState as FigConversationState,
    EnvState,
    EnvironmentVariable,
    GitState,
    ShellHistoryEntry,
    ShellState,
    Tool,
    ToolInputSchema,
    ToolResult,
    ToolResultContentBlock,
    ToolSpecification,
    UserInputMessage,
    UserInputMessageContext,
};
use fig_settings::history::{
    History,
    OrderBy,
};
use fig_util::Shell;
use regex::Regex;
use tracing::{
    error,
    warn,
};

use crate::cli::chat::ToolConfiguration;
use crate::cli::chat::tools::{
    InputSchema,
    InvokeOutput,
    serde_value_to_document,
};

// Max constants for length of strings and lists, use these to truncate elements
// to ensure the API request is valid

// These limits are the internal undocumented values from the service for each item

const MAX_ENV_VAR_LIST_LEN: usize = 100;
const MAX_ENV_VAR_KEY_LEN: usize = 256;
const MAX_ENV_VAR_VALUE_LEN: usize = 1024;
const MAX_CURRENT_WORKING_DIRECTORY_LEN: usize = 256;

const MAX_GIT_STATUS_LEN: usize = 4096;

const MAX_SHELL_HISTORY_LIST_LEN: usize = 20;
const MAX_SHELL_HISTORY_COMMAND_LEN: usize = 1024;
const MAX_SHELL_HISTORY_DIRECTORY_LEN: usize = 256;

/// Regex for the context modifiers `@git`, `@env`, and `@history`
static CONTEXT_MODIFIER_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"@(git|env|history) ?").unwrap());

/// Limit to send the number of messages as part of chat.
const MAX_CONVERSATION_STATE_HISTORY_LEN: usize = 25;

/// Tracks state related to an ongoing conversation.
#[derive(Debug, Clone)]
pub struct ConversationState {
    pub conversation_id: Option<String>,
    /// The next user message to be sent as part of the conversation. Required to be [Some] before
    /// calling [Self::as_sendable_conversation_state].
    pub next_message: Option<Message>,
    pub history: VecDeque<Message>,
    tools: Vec<Tool>,
}

impl ConversationState {
    pub fn new(tool_config: ToolConfiguration) -> Self {
        Self {
            conversation_id: None,
            next_message: None,
            history: VecDeque::new(),
            tools: tool_config
                .tools
                .into_values()
                .map(|v| {
                    Tool::ToolSpecification(ToolSpecification {
                        name: v.name,
                        description: v.description,
                        input_schema: v.input_schema.into(),
                    })
                })
                .collect(),
        }
    }

    /// Clears the conversation history.
    pub fn clear(&mut self) {
        self.next_message = None;
        self.history.clear();
    }

    pub async fn append_new_user_message(&mut self, input: String) {
        debug_assert!(self.next_message.is_none(), "next_message should not exist");
        if let Some(next_message) = self.next_message.as_ref() {
            warn!(?next_message, "next_message should not exist");
        }
        let (ctx, input) = input_to_modifiers(input);
        let history = History::new();

        let mut user_input_message_context = UserInputMessageContext {
            shell_state: Some(build_shell_state(ctx.history, &history)),
            env_state: Some(build_env_state(Some(&ctx))),
            tool_results: None,
            tools: if self.tools.is_empty() {
                None
            } else {
                Some(self.tools.clone())
            },
            ..Default::default()
        };

        if ctx.git {
            if let Ok(git_state) = build_git_state(None).await {
                user_input_message_context.git_state = Some(git_state);
            }
        }

        let msg = Message(ChatMessage::UserInputMessage(UserInputMessage {
            content: input,
            user_input_message_context: Some(user_input_message_context),
            user_intent: None,
        }));
        self.next_message = Some(msg);
    }

    pub fn push_assistant_message(&mut self, message: Message) {
        debug_assert!(self.next_message.is_none(), "next_message should not exist");
        if let Some(next_message) = self.next_message.as_ref() {
            warn!(?next_message, "next_message should not exist");
        }
        self.history.push_back(message);
    }

    /// Returns the conversation id, if available.
    pub fn conversation_id(&self) -> Option<&str> {
        self.conversation_id.as_deref()
    }

    /// Returns the message id associated with the last assistant message, if present.
    pub fn message_id(&self) -> Option<&str> {
        self.history.iter().last().and_then(|m| match &m.0 {
            ChatMessage::AssistantResponseMessage(m) => m.message_id.as_deref(),
            ChatMessage::UserInputMessage(_) => None,
        })
    }

    pub fn add_tool_results(&mut self, tool_results: Vec<ToolResult>) {
        let user_input_message_context = UserInputMessageContext {
            shell_state: None,
            env_state: Some(build_env_state(None)),
            tool_results: Some(tool_results),
            tools: if self.tools.is_empty() {
                None
            } else {
                Some(self.tools.clone())
            },
            ..Default::default()
        };
        let msg = Message(ChatMessage::UserInputMessage(UserInputMessage {
            content: String::new(),
            user_input_message_context: Some(user_input_message_context),
            user_intent: None,
        }));
        self.next_message = Some(msg);
    }

    pub fn abandon_tool_use(&mut self, tools_to_be_abandoned: Vec<(String, super::tools::Tool)>, deny_input: String) {
        let tool_results = tools_to_be_abandoned
            .into_iter()
            .map(|(tool_use_id, _)| ToolResult {
                tool_use_id,
                content: vec![ToolResultContentBlock::Text(
                    "Tool use was cancelled by the user".to_string(),
                )],
                status: fig_api_client::model::ToolResultStatus::Error,
            })
            .collect::<Vec<_>>();
        let user_input_message_context = UserInputMessageContext {
            shell_state: None,
            env_state: Some(build_env_state(None)),
            tool_results: Some(tool_results),
            tools: if self.tools.is_empty() {
                None
            } else {
                Some(self.tools.clone())
            },
            ..Default::default()
        };
        let msg = Message(ChatMessage::UserInputMessage(UserInputMessage {
            content: deny_input,
            user_input_message_context: Some(user_input_message_context),
            user_intent: None,
        }));
        self.next_message = Some(msg);
    }

    /// Returns a [FigConversationState] capable of being sent by
    /// [fig_api_client::StreamingClient] while preparing the current conversation state to be sent
    /// in the next message.
    pub fn as_sendable_conversation_state(&mut self) -> FigConversationState {
        assert!(self.next_message.is_some());
        while self.history.len() > MAX_CONVERSATION_STATE_HISTORY_LEN {
            self.history.pop_front();
        }

        // The current state we want to send
        let curr_state = self.clone();

        // Updating `self` so that the current next_message is moved to history.
        let mut last_message = self.next_message.take().unwrap();
        match &mut last_message.0 {
            ChatMessage::UserInputMessage(msg) => {
                if let Some(ctx) = &mut msg.user_input_message_context {
                    ctx.tools.take();
                }
            },
            ChatMessage::AssistantResponseMessage(_) => (),
        }
        self.history.push_back(last_message);

        FigConversationState {
            conversation_id: curr_state.conversation_id,
            user_input_message: curr_state
                .next_message
                .and_then(|m| match m.0 {
                    ChatMessage::AssistantResponseMessage(_) => None,
                    ChatMessage::UserInputMessage(user_input_message) => Some(user_input_message),
                })
                .expect("no user input message available"),
            history: Some(curr_state.history.into_iter().map(|m| m.0).collect()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Message(pub ChatMessage);

impl From<InvokeOutput> for ToolResultContentBlock {
    fn from(value: InvokeOutput) -> Self {
        match value.output {
            crate::cli::chat::tools::OutputKind::Text(text) => Self::Text(text),
            crate::cli::chat::tools::OutputKind::Json(value) => Self::Json(serde_value_to_document(value)),
        }
    }
}

impl From<InputSchema> for ToolInputSchema {
    fn from(value: InputSchema) -> Self {
        Self {
            json: Some(serde_value_to_document(value.0)),
        }
    }
}

/// The context modifiers that are used in a specific chat message
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ContextModifiers {
    env: bool,
    history: bool,
    git: bool,
}

impl ContextModifiers {
    /// Returns `true` if any context modifiers are set
    #[allow(dead_code)]
    fn any(&self) -> bool {
        self.env || self.history || self.git
    }
}

/// Convert the `input` into the [ContextModifiers] and a string with them removed
fn input_to_modifiers(input: String) -> (ContextModifiers, String) {
    let mut modifiers = ContextModifiers::default();

    for capture in CONTEXT_MODIFIER_REGEX.captures_iter(&input) {
        let modifier = capture.get(1).expect("regex has a capture group").as_str();

        match modifier {
            "git" => modifiers.git = true,
            "env" => modifiers.env = true,
            "history" => modifiers.history = true,
            _ => unreachable!(),
        }
    }

    (modifiers, input)
}

async fn build_git_state(dir: Option<&Path>) -> Result<GitState> {
    // git status --porcelain=v1 -b
    let mut command = tokio::process::Command::new("git");
    command.args(["status", "--porcelain=v1", "-b"]);
    if let Some(dir) = dir {
        command.current_dir(dir);
    }
    let output = command.output().await?;

    if output.status.success() && !output.stdout.is_empty() {
        Ok(GitState {
            status: truncate_safe(&String::from_utf8_lossy(&output.stdout), MAX_GIT_STATUS_LEN).into(),
        })
    } else {
        bail!("git status failed: {output:?}")
    }
}

fn build_env_state(modifiers: Option<&ContextModifiers>) -> EnvState {
    let mut env_state = EnvState {
        operating_system: Some(env::consts::OS.into()),
        ..Default::default()
    };

    match env::current_dir() {
        Ok(current_dir) => {
            env_state.current_working_directory =
                Some(truncate_safe(&current_dir.to_string_lossy(), MAX_CURRENT_WORKING_DIRECTORY_LEN).into());
        },
        Err(err) => {
            error!(?err, "Attempted to fetch the CWD but it did not exist.");
        },
    }

    if modifiers.is_some_and(|c| c.env) {
        for (key, value) in env::vars().take(MAX_ENV_VAR_LIST_LEN) {
            if !key.is_empty() && !value.is_empty() {
                env_state.environment_variables.push(EnvironmentVariable {
                    key: truncate_safe(&key, MAX_ENV_VAR_KEY_LEN).into(),
                    value: truncate_safe(&value, MAX_ENV_VAR_VALUE_LEN).into(),
                });
            }
        }
    }

    env_state
}

fn build_shell_state(shell_history: bool, history: &History) -> ShellState {
    // Try to grab the shell from the parent process via the `Shell::current_shell`,
    // then try the `SHELL` env, finally just report bash
    let shell_name = Shell::current_shell()
        .or_else(|| {
            let shell_name = env::var("SHELL").ok()?;
            Shell::try_find_shell(shell_name)
        })
        .unwrap_or(Shell::Bash)
        .to_string();

    let mut shell_state = ShellState {
        shell_name,
        shell_history: None,
    };

    if shell_history {
        shell_state.shell_history = build_shell_history(history);
    }

    shell_state
}

fn build_shell_history(history: &History) -> Option<Vec<ShellHistoryEntry>> {
    let mut shell_history = vec![];

    if let Ok(commands) = history.rows(
        None,
        vec![OrderBy::new(
            fig_settings::history::HistoryColumn::Id,
            fig_settings::history::Order::Desc,
        )],
        MAX_SHELL_HISTORY_LIST_LEN,
        0,
    ) {
        for command in commands.into_iter().filter(|c| c.command.is_some()).rev() {
            let command_str = command.command.expect("command is filtered on");
            if !command_str.is_empty() {
                shell_history.push(ShellHistoryEntry {
                    command: truncate_safe(&command_str, MAX_SHELL_HISTORY_COMMAND_LEN).into(),
                    directory: command.cwd.and_then(|cwd| {
                        if !cwd.is_empty() {
                            Some(truncate_safe(&cwd, MAX_SHELL_HISTORY_DIRECTORY_LEN).into())
                        } else {
                            None
                        }
                    }),
                    exit_code: command.exit_code,
                });
            }
        }
    }

    if shell_history.is_empty() {
        None
    } else {
        Some(shell_history)
    }
}

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

#[cfg(test)]
mod tests {
    use fig_api_client::model::AssistantResponseMessage;
    use fig_settings::history::CommandInfo;

    use super::*;
    use crate::cli::chat::load_tools;

    #[test]
    fn test_truncate_safe() {
        assert_eq!(truncate_safe("Hello World", 5), "Hello");
        assert_eq!(truncate_safe("Hello ", 5), "Hello");
        assert_eq!(truncate_safe("Hello World", 11), "Hello World");
        assert_eq!(truncate_safe("Hello World", 15), "Hello World");
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
        assert_eq!(input, "@git @env @history How do I use git?");

        let (modifiers, input) = input_to_modifiers("@git How do I use git?".to_string());
        assert_eq!(modifiers, ContextModifiers {
            env: false,
            history: false,
            git: true
        });
        assert_eq!(input, "@git How do I use git?");

        let (modifiers, input) = input_to_modifiers("@env How do I use git?".to_string());
        assert_eq!(modifiers, ContextModifiers {
            env: true,
            history: false,
            git: false
        });
        assert_eq!(input, "@env How do I use git?");
    }

    #[test]
    fn test_shell_state() {
        let history = History::mock();
        history
            .insert_command_history(
                &CommandInfo {
                    command: Some("ls".into()),
                    cwd: Some("/home/user".into()),
                    exit_code: Some(0),
                    ..Default::default()
                },
                false,
            )
            .unwrap();

        let shell_state = build_shell_state(true, &history);

        for ShellHistoryEntry {
            command,
            directory,
            exit_code,
        } in shell_state.shell_history.unwrap()
        {
            println!("{command} {directory:?} {exit_code:?}");
        }
    }

    #[test]
    fn test_env_state() {
        // env: true
        let env_state = build_env_state(Some(&ContextModifiers {
            env: true,
            history: false,
            git: false,
        }));
        assert!(!env_state.environment_variables.is_empty());
        assert!(!env_state.current_working_directory.as_ref().unwrap().is_empty());
        assert!(!env_state.operating_system.as_ref().unwrap().is_empty());
        println!("{env_state:?}");

        // env: false
        let env_state = build_env_state(Some(&ContextModifiers::default()));
        assert!(env_state.environment_variables.is_empty());
        assert!(env_state.current_working_directory.is_some());
        assert!(env_state.operating_system.as_ref().is_some_and(|os| !os.is_empty()));
        println!("{env_state:?}");
    }

    async fn init_git_repo(dir: &Path) {
        let output = tokio::process::Command::new("git")
            .arg("init")
            .current_dir(dir)
            .output()
            .await
            .unwrap();
        println!("== git init ==");
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        assert!(output.status.success());
    }

    #[tokio::test]
    async fn test_git_state() {
        let tempdir = tempfile::tempdir().unwrap();
        let dir_path = tempdir.path();

        // write a file to the repo to ensure git status has a change
        let path = dir_path.join("test.txt");
        std::fs::write(path, "test").unwrap();

        let git_state_err = build_git_state(Some(dir_path)).await.unwrap_err();
        println!("git_state_err: {git_state_err}");
        assert!(git_state_err.to_string().contains("git status failed"));

        init_git_repo(dir_path).await;

        let git_state = build_git_state(Some(dir_path)).await.unwrap();
        println!("git_state: {git_state:?}");
    }

    #[tokio::test]
    async fn test_conversation_state_history_handling() {
        let mut conversation_state = ConversationState::new(load_tools().unwrap());

        conversation_state.append_new_user_message("start".to_string()).await;
        for i in 0..=20 {
            let _ = conversation_state.as_sendable_conversation_state();
            conversation_state.push_assistant_message(Message(ChatMessage::AssistantResponseMessage(
                AssistantResponseMessage {
                    message_id: None,
                    content: i.to_string(),
                    tool_uses: None,
                },
            )));
            conversation_state.append_new_user_message(i.to_string()).await;
        }

        let s = conversation_state.as_sendable_conversation_state();
        assert_eq!(
            s.history.as_ref().unwrap().len(),
            MAX_CONVERSATION_STATE_HISTORY_LEN,
            "history should be capped at {}",
            MAX_CONVERSATION_STATE_HISTORY_LEN
        );
        let last_msg = s.history.as_ref().unwrap().iter().last().unwrap();
        match last_msg {
            ChatMessage::AssistantResponseMessage(assistant_response_message) => {
                assert_eq!(assistant_response_message.content, "20");
            },
            other @ ChatMessage::UserInputMessage(_) => {
                panic!("Last message should be from the assistant, instead found {:?}", other)
            },
        }
    }
}
