use anyhow::Result;
use fig_proto::local::{CursorPositionHook, EditBufferHook, InitHook};

use crate::state::AppStateType;

pub async fn init(_state: &AppStateType, _hook: InitHook) -> Result<()> {
    todo!()
}

pub async fn edit_buffer(_state: &AppStateType, _hook: EditBufferHook) -> Result<()> {
    todo!()
}

pub async fn cursor_position(_state: &AppStateType, _hook: CursorPositionHook) -> Result<()> {
    todo!()
}
