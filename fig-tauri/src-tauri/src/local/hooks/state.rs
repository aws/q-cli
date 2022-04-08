use anyhow::Result;
use fig_proto::local::{CursorPositionHook, EditBufferHook, PromptHook};

use crate::{local::figterm::ensure_figterm, state::STATE};

pub async fn edit_buffer(hook: EditBufferHook) -> Result<()> {
    ensure_figterm(hook.context.unwrap().session_id.unwrap());
    let mut handle = STATE.lock();
    handle.edit_buffer.text = hook.text;
    handle.edit_buffer.cursor = hook.cursor;
    Ok(())
}

pub async fn cursor_position(hook: CursorPositionHook) -> Result<()> {
    let mut handle = STATE.lock();
    handle.cursor_position.x = hook.x;
    handle.cursor_position.y = hook.y;
    handle.cursor_position.width = hook.width;
    handle.cursor_position.height = hook.height;
    Ok(())
}

pub async fn prompt(hook: PromptHook) -> Result<()> {
    ensure_figterm(hook.context.unwrap().session_id.unwrap());
    Ok(())
}
