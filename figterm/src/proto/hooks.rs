use crate::proto;

/// Construct a new Shell Context
pub fn new_context(
    pid: Option<i32>,
    ttys: Option<String>,
    process_name: Option<String>,
    current_working_directory: Option<String>,
    session_id: Option<String>,
    integration_version: Option<i32>,
    terminal: Option<String>,
    hostname: Option<String>,
) -> proto::ShellContext {
    proto::ShellContext {
        pid,
        ttys,
        process_name: None,
        current_working_directory: None,
        session_id,
        integration_version,
        terminal: None,
        hostname: None,
        remote_context: None,
    }
}

/// Construct a edit buffer hook
pub fn new_edit_buffer_hook(
    context: Option<proto::ShellContext>,
    text: String,
    cursor: i64,
    histno: i64,
) -> proto::hook::Hook {
    proto::hook::Hook::EditBuffer(proto::EditBufferHook {
        context,
        text,
        cursor,
        histno,
    })
}

pub fn hook_to_message(hook: proto::hook::Hook) -> proto::LocalMessage {
    proto::LocalMessage {
        r#type: Some(proto::local_message::Type::Hook(proto::local::Hook {
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
