//! Wrappers and re-exports around the Bedrock SDK types.
//!
//! This is intended to make refactoring to the actual client a bit more straightforward - we
//! should ideally only need to make updates to this file rather than all of the tool/parsing/CLI
//! files in tandem.

pub mod bedrock;
pub use bedrock::*;
use fig_settings::history::{
    History,
    OrderBy,
};
pub mod q;
// pub use q::*;

use std::env;
use std::path::Path;
use std::sync::LazyLock;

use eyre::{
    Result,
    bail,
};
use fig_api_client::model::{
    EnvState,
    EnvironmentVariable,
    GitState,
    ShellHistoryEntry,
    ShellState,
    UserIntent,
};
use fig_util::Shell;
use regex::Regex;

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

#[derive(Debug, Clone)]
pub enum ConversationRole {
    User,
    Assistant,
}

#[derive(Debug, Clone)]
pub enum StopReason {
    EndTurn,
    ToolUse,
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
    fn any(&self) -> bool {
        self.env || self.history || self.git
    }

    /// Returns a [`UserIntent`] that disables RAG if any context modifiers are set
    #[allow(clippy::unused_self)]
    fn user_intent(&self) -> Option<UserIntent> {
        // disabled while user intents all change prompt
        // if self.any() {
        //     Some(UserIntent::ApplyCommonBestPractices)
        // } else {
        //     None
        // }
        None
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

fn build_env_state(modifiers: &ContextModifiers) -> EnvState {
    let mut env_state = EnvState {
        operating_system: Some(env::consts::OS.into()),
        ..Default::default()
    };

    if modifiers.any() {
        if let Ok(current_dir) = env::current_dir() {
            env_state.current_working_directory =
                Some(truncate_safe(&current_dir.to_string_lossy(), MAX_CURRENT_WORKING_DIRECTORY_LEN).into());
        }
    }

    if modifiers.env {
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
    use fig_settings::history::CommandInfo;

    use super::*;

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
        let env_state = build_env_state(&ContextModifiers {
            env: true,
            history: false,
            git: false,
        });
        assert!(!env_state.environment_variables.is_empty());
        assert!(!env_state.current_working_directory.as_ref().unwrap().is_empty());
        assert!(!env_state.operating_system.as_ref().unwrap().is_empty());
        println!("{env_state:?}");

        // env: false
        let env_state = build_env_state(&ContextModifiers::default());
        assert!(env_state.environment_variables.is_empty());
        assert!(env_state.current_working_directory.is_none());
        assert!(!env_state.operating_system.as_ref().unwrap().is_empty());
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
}
