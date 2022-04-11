use anyhow::Result;
use fig_proto::local::{CursorPositionHook, EditBufferHook, PromptHook};

use crate::{local::figterm::ensure_figterm, state::STATE};

pub async fn edit_buffer(hook: EditBufferHook) -> Result<()> {
    let session_id = hook.context.unwrap().session_id.unwrap();
    ensure_figterm(session_id.clone());
    let mut session = STATE.figterm_sessions.get_mut(&session_id).unwrap();
    session.edit_buffer.text = hook.text;
    session.edit_buffer.cursor = hook.cursor;
    Ok(())
}

pub async fn cursor_position(hook: CursorPositionHook) -> Result<()> {
    let mut handle = STATE.cursor_position.lock();
    handle.x = hook.x;
    handle.y = hook.y;
    handle.width = hook.width;
    handle.height = hook.height;
    Ok(())
}

pub async fn prompt(_: PromptHook) -> Result<()> {
    Ok(())
}
