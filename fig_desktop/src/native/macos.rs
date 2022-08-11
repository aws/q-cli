use std::sync::Arc;

use anyhow::Result;

use crate::event::NativeEvent;
use crate::{
    EventLoopProxy,
    GlobalState,
};

pub const SHELL: &str = "/bin/bash";

#[derive(Debug, Default)]
pub struct NativeState {}

impl NativeState {
    pub fn handle(&self, _event: NativeEvent) -> Result<()> {
        Ok(())
    }
}

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
