use std::sync::Arc;

use anyhow::Result;

use crate::window::CursorPositionKind;
use crate::{
    EventLoopProxy,
    GlobalState,
};

pub const SHELL: &str = "/bin/bash";
pub const SHELL_ARGS: [&str; 3] = ["--noprofile", "--norc", "-c"];
pub const CURSOR_POSITION_KIND: CursorPositionKind = CursorPositionKind::Relative;

#[derive(Debug, Default)]
pub struct NativeState {}

pub async fn init(_global_state: Arc<GlobalState>, _proxy: EventLoopProxy) -> Result<()> {
    Ok(())
}

pub mod icons {
    use crate::icons::ProcessedAsset;

    #[allow(unused_variables)]
    pub fn lookup(name: &str) -> Option<ProcessedAsset> {
        None
    }
}
