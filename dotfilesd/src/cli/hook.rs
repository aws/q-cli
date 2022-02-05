use crate::ipc::{
    hook::{
        create_edit_buffer_hook, create_event_hook, create_hide_hook, create_init_hook,
        create_integration_ready_hook, create_keyboard_focus_changed_hook, create_preexec_hook,
        create_prompt_hook,
    },
    send_hook_to_socket,
};
use anyhow::{Context, Result};
use clap::Subcommand;
use crossterm::style::Stylize;

#[derive(Debug, Subcommand)]
pub enum HookSubcommand {
    Editbuffer {
        session_id: String,
        integration: i32,
        tty: String,
        pid: i32,
        histno: i32,
        cursor: i32,
        text: String,
    },
    Event {
        event_name: String,
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
}

impl HookSubcommand {
    pub async fn execute(&self) -> Result<()> {
        let hook = match self {
            HookSubcommand::Editbuffer {
                session_id,
                integration,
                tty,
                pid,
                histno,
                cursor,
                text,
            } => create_edit_buffer_hook(
                session_id,
                *integration,
                tty,
                *pid,
                i64::from(*histno),
                i64::from(*cursor),
                text,
            ),
            HookSubcommand::Event { event_name } => create_event_hook(event_name),
            HookSubcommand::Hide => create_hide_hook(),
            HookSubcommand::Init { pid, tty } => create_init_hook(*pid, tty),
            HookSubcommand::IntegrationReady { integration } => {
                create_integration_ready_hook(integration)
            }
            HookSubcommand::KeyboardFocusChanged {
                app_identifier,
                focused_session_id,
            } => create_keyboard_focus_changed_hook(app_identifier, focused_session_id),
            HookSubcommand::PreExec { pid, tty } => create_preexec_hook(*pid, tty),
            HookSubcommand::Prompt { pid, tty } => create_prompt_hook(*pid, tty),
        };
        let hook = hook.context("Invalid input for hook")?;
        send_hook_to_socket(hook).await.context(format!(
            "\n{}\nFig might not be running to launch Fig run: {}\n",
            "Unable to Connect to Fig:".bold(),
            "fig launch".magenta()
        ))
    }
}
