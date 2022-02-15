use crate::local;

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
) -> local::ShellContext {
    local::ShellContext {
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

/// Construct a edit buffer hook
pub fn new_edit_buffer_hook(
    context: Option<local::ShellContext>,
    text: String,
    cursor: i64,
    histno: i64,
) -> local::hook::Hook {
    local::hook::Hook::EditBuffer(local::EditBufferHook {
        context,
        text,
        cursor,
        histno,
    })
}

/// Construct a new prompt hook
pub fn new_prompt_hook(context: Option<local::ShellContext>) -> local::hook::Hook {
    local::hook::Hook::Prompt(local::PromptHook { context })
}

pub fn new_preexec_hook(context: Option<local::ShellContext>) -> local::hook::Hook {
    local::hook::Hook::PreExec(local::PreExecHook {
        context,
        command: None,
    })
}

pub fn hook_to_message(hook: local::hook::Hook) -> local::LocalMessage {
    local::LocalMessage {
        r#type: Some(local::local_message::Type::Hook(local::Hook {
            hook: Some(hook),
        })),
    }
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
        new_edit_buffer_hook(None, "test".into(), 0, 0);
    }
}
