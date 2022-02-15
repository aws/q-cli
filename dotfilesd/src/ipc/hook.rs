use crate::util::get_shell;
use anyhow::Result;
use fig_proto::local::{
    hook::Hook, EditBufferHook, EventHook, HideHook, InitHook, IntegrationReadyHook,
    KeyboardFocusChangedHook, PreExecHook, PromptHook, ShellContext,
};
use std::collections::HashMap;

const CURRENT_INTEGRATION_VERSION: i32 = 5;

fn generate_shell_context(
    pid: i32,
    tty: impl Into<String>,
    session_id: impl Into<Option<String>>,
    integration_version: Option<i32>,
) -> Result<ShellContext> {
    let cwd = std::env::current_dir()?;
    let shell = get_shell()?;
    Ok(ShellContext {
        pid: Some(pid),
        ttys: Some(tty.into()),
        session_id: session_id
            .into()
            .or_else(|| std::env::var("TERM_SESSION_ID").ok()),
        integration_version: Some(integration_version.unwrap_or(CURRENT_INTEGRATION_VERSION)),
        process_name: Some(shell),
        current_working_directory: Some(cwd.to_string_lossy().into()),
        terminal: None,
        hostname: None,
        remote_context: None,
    })
}

pub fn create_edit_buffer_hook(
    session_id: impl Into<String>,
    integration_version: i32,
    tty: impl Into<String>,
    pid: i32,
    histno: i64,
    cursor: i64,
    text: impl Into<String>,
) -> Result<Hook> {
    let context =
        generate_shell_context(pid, tty, Some(session_id.into()), Some(integration_version))?;
    Ok(Hook::EditBuffer(EditBufferHook {
        context: Some(context),
        text: text.into(),
        cursor,
        histno,
    }))
}

pub fn create_prompt_hook(pid: i32, tty: impl Into<String>) -> Result<Hook> {
    let context = generate_shell_context(pid, tty, None, None)?;
    Ok(Hook::Prompt(PromptHook {
        context: Some(context),
    }))
}

pub fn create_init_hook(pid: i32, tty: impl Into<String>) -> Result<Hook> {
    let env_map: HashMap<_, _> = std::env::vars().collect();
    let context = generate_shell_context(pid, tty, None, None)?;
    Ok(Hook::Init(InitHook {
        context: Some(context),
        called_direct: false,
        bundle: "".into(), // GetCurrentTerminal()?.PotentialBundleId()?
        env: env_map,
    }))
}

pub fn create_keyboard_focus_changed_hook(
    app_identifier: impl Into<String>,
    focused_session_id: impl Into<String>,
) -> Result<Hook> {
    Ok(Hook::KeyboardFocusChanged(KeyboardFocusChangedHook {
        app_identifier: app_identifier.into(),
        focused_session_id: focused_session_id.into(),
    }))
}

pub fn create_integration_ready_hook(identifier: impl Into<String>) -> Result<Hook> {
    Ok(Hook::IntegrationReady(IntegrationReadyHook {
        identifier: identifier.into(),
    }))
}

pub fn create_hide_hook() -> Result<Hook> {
    Ok(Hook::Hide(HideHook {}))
}

pub fn create_event_hook(event_name: impl Into<String>) -> Result<Hook> {
    Ok(Hook::Event(EventHook {
        event_name: event_name.into(),
    }))
}

pub fn create_preexec_hook(pid: i32, tty: impl Into<String>) -> Result<Hook> {
    let context = generate_shell_context(pid, tty, None, None)?;
    Ok(Hook::PreExec(PreExecHook {
        context: Some(context),
        command: None,
    }))
}
