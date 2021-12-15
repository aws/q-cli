use crate::proto;

pub fn new_context(
    session_id: Option<String>,
    pid: Option<i32>,
    ttys: Option<String>,
    integration_version: Option<i32>,
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
