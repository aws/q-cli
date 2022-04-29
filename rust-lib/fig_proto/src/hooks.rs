use std::collections::HashMap;

use crate::{local::*, util::get_shell};
use anyhow::Result;

const CURRENT_INTEGRATION_VERSION: i32 = 7;

/// Construct a new Shell Context
#[allow(clippy::too_many_arguments)]
pub fn new_context(
    pid: Option<i32>,
    ttys: Option<String>,
    process_name: Option<String>,
    current_working_directory: Option<String>,
    session_id: Option<String>,
    integration_version: Option<i32>,
    terminal: Option<String>,
    hostname: Option<String>,
) -> ShellContext {
    ShellContext {
        pid,
        ttys,
        process_name,
        current_working_directory,
        session_id,
        integration_version,
        terminal,
        hostname,
        remote_context: None,
    }
}

fn hook_enum_to_hook(hook: hook::Hook) -> Hook {
    Hook { hook: Some(hook) }
}

pub fn hook_to_message(hook: Hook) -> LocalMessage {
    LocalMessage {
        r#type: Some(local_message::Type::Hook(hook)),
    }
}

pub fn generate_shell_context(
    pid: impl Into<i32>,
    tty: impl Into<String>,
    session_id: impl Into<Option<String>>,
    integration_version: impl Into<Option<i32>>,
) -> Result<ShellContext> {
    let cwd = std::env::current_dir()?;
    let shell = get_shell()?;
    Ok(ShellContext {
        pid: Some(pid.into()),
        ttys: Some(tty.into()),
        session_id: session_id
            .into()
            .or_else(|| std::env::var("TERM_SESSION_ID").ok()),
        integration_version: Some(
            integration_version
                .into()
                .unwrap_or(CURRENT_INTEGRATION_VERSION),
        ),
        process_name: Some(shell),
        current_working_directory: Some(cwd.to_string_lossy().into()),
        terminal: None,
        hostname: None,
        remote_context: None,
    })
}

/// Construct a edit buffer hook
pub fn new_edit_buffer_hook(
    context: impl Into<Option<ShellContext>>,
    text: impl Into<String>,
    cursor: i64,
    histno: i64,
) -> Hook {
    hook_enum_to_hook(hook::Hook::EditBuffer(EditBufferHook {
        context: context.into(),
        text: text.into(),
        cursor,
        histno,
    }))
}

/// Construct a new hook
pub fn new_init_hook(context: impl Into<Option<ShellContext>>) -> Result<Hook> {
    let env_map: HashMap<_, _> = std::env::vars().collect();

    Ok(hook_enum_to_hook(hook::Hook::Init(InitHook {
        context: context.into(),
        called_direct: false,
        bundle: "".into(), // GetCurrentTerminal()?.PotentialBundleId()?
        env: env_map,
    })))
}

/// Construct a new prompt hook
pub fn new_prompt_hook(context: impl Into<Option<ShellContext>>) -> Hook {
    hook_enum_to_hook(hook::Hook::Prompt(PromptHook {
        context: context.into(),
    }))
}

pub fn new_preexec_hook(context: impl Into<Option<ShellContext>>) -> Hook {
    hook_enum_to_hook(hook::Hook::PreExec(PreExecHook {
        context: context.into(),
        command: None,
    }))
}

pub fn new_keyboard_focus_changed_hook(
    app_identifier: impl Into<String>,
    focused_session_id: impl Into<String>,
) -> Hook {
    hook_enum_to_hook(hook::Hook::KeyboardFocusChanged(KeyboardFocusChangedHook {
        app_identifier: app_identifier.into(),
        focused_session_id: focused_session_id.into(),
    }))
}

pub fn new_ssh_hook(
    context: impl Into<Option<ShellContext>>,
    control_path: impl Into<String>,
    remote_dest: impl Into<String>,
) -> Result<Hook> {
    Ok(hook_enum_to_hook(hook::Hook::OpenedSshConnection(
        OpenedSshConnectionHook {
            context: context.into(),
            control_path: control_path.into(),
            remote_hostname: remote_dest.into(),
        },
    )))
}

pub fn new_integration_ready_hook(identifier: impl Into<String>) -> Hook {
    hook_enum_to_hook(hook::Hook::IntegrationReady(IntegrationReadyHook {
        identifier: identifier.into(),
    }))
}

pub fn new_hide_hook() -> Hook {
    hook_enum_to_hook(hook::Hook::Hide(HideHook {}))
}

pub fn new_event_hook(event_name: impl Into<String>) -> Hook {
    hook_enum_to_hook(hook::Hook::Event(EventHook {
        event_name: event_name.into(),
    }))
}

pub fn new_file_changed_hook(
    file_changed: file_changed_hook::FileChanged,
    filepath: impl Into<Option<String>>,
) -> Hook {
    hook_enum_to_hook(hook::Hook::FileChanged(FileChangedHook {
        file_changed: file_changed.into(),
        filepath: filepath.into(),
    }))
}

pub fn new_callback_hook(
    handler_id: impl Into<String>,
    filepath: impl Into<String>,
    exit_code: i64,
) -> Hook {
    hook_enum_to_hook(hook::Hook::Callback(CallbackHook {
        handler_id: handler_id.into(),
        filepath: filepath.into(),
        exit_code: exit_code.to_string(),
    }))
}

pub fn new_intercepted_key_hook(
    context: impl Into<Option<ShellContext>>,
    action: impl Into<String>,
    key: impl Into<String>,
) -> Hook {
    hook_enum_to_hook(hook::Hook::InterceptedKey(InterceptedKeyHook {
        context: context.into(),
        action: action.into(),
        key: key.into(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_contest_test() {
        new_context(None, None, None, None, None, None, None, None);
    }

    #[test]
    fn new_edit_buffer_hook_test() {
        new_edit_buffer_hook(None, "test", 0, 0);
    }
}
