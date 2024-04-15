use std::process::exit;

use clap::Subcommand;
use eyre::{
    Result,
    WrapErr,
};
use fig_ipc::local::send_hook_to_socket;
use fig_proto::hooks;

#[derive(Debug, PartialEq, Eq, Subcommand)]
#[command(hide = true)]
pub enum HookSubcommand {
    Editbuffer {
        session_id: String,
        integration: i32,
        tty: String,
        pid: i32,
        histno: i64,
        cursor: i64,
        text: String,
    },
    Hide,
    Init {
        pid: i32,
        tty: String,
    },
    IntegrationReady {
        integration: String,
    },
    KeyboardFocusChanged {
        app_identifier: String,
        focused_session_id: String,
    },
    PreExec {
        pid: i32,
        tty: String,
    },
    Prompt {
        pid: i32,
        tty: String,
    },
    Ssh {
        pid: i32,
        tty: String,
        control_path: String,
        remote_dest: String,
        #[arg(long)]
        prompt: bool,
    },
    ClearAutocompleteCache {
        #[arg(long)]
        cli: Vec<String>,
    },
}

impl HookSubcommand {
    pub async fn execute(&self) -> Result<()> {
        // Hooks should exit silently on failure.
        match self.execute_hook().await {
            Ok(()) => Ok(()),
            Err(_) => exit(1),
        }
    }

    pub async fn execute_hook(&self) -> Result<()> {
        let session_id = std::env::var("CWTERM_SESSION_ID").ok();

        let hook = match self {
            HookSubcommand::Editbuffer {
                session_id,
                tty,
                pid,
                histno,
                cursor,
                text,
                ..
            } => {
                let context = hooks::generate_shell_context(*pid, tty, Some(session_id.clone()))?;
                Ok(hooks::new_edit_buffer_hook(context, text, *histno, *cursor, None))
            },
            HookSubcommand::Hide => Ok(hooks::new_hide_hook()),
            HookSubcommand::Init { pid, tty } => {
                let context = hooks::generate_shell_context(*pid, tty, session_id)?;
                hooks::new_init_hook(context)
            },
            HookSubcommand::IntegrationReady { integration } => Ok(hooks::new_integration_ready_hook(integration)),
            HookSubcommand::KeyboardFocusChanged {
                app_identifier,
                focused_session_id,
            } => Ok(hooks::new_keyboard_focus_changed_hook(
                app_identifier,
                focused_session_id,
            )),
            HookSubcommand::PreExec { pid, tty } => {
                let context = hooks::generate_shell_context(*pid, tty, session_id)?;
                Ok(hooks::new_preexec_hook(context))
            },
            HookSubcommand::Prompt { pid, tty } => {
                let context = hooks::generate_shell_context(*pid, tty, session_id)?;
                Ok(hooks::new_prompt_hook(context))
            },
            HookSubcommand::Ssh {
                control_path,
                pid,
                tty,
                remote_dest,
                ..
            } => {
                let context = hooks::generate_shell_context(*pid, tty, session_id)?;
                hooks::new_ssh_hook(context, control_path, remote_dest)
            },
            HookSubcommand::ClearAutocompleteCache { cli } => Ok(hooks::new_clear_autocomplete_cache(cli.clone())),
        };

        let hook = hook.context("Invalid input for hook")?;
        Ok(send_hook_to_socket(hook).await?)
    }
}
