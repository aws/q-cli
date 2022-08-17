use std::sync::Arc;

use anyhow::Result;

use crate::event::NativeEvent;
use crate::EventLoopProxy;

pub const SHELL: &str = "/bin/bash";

#[derive(Debug)]
pub struct NativeState {}

impl NativeState {
    pub fn new(_proxy: EventLoopProxy) -> Self {
        Self {}
    }

    pub fn handle(&self, _event: NativeEvent) -> Result<()> {
        Ok(())
    }
}

pub async fn init(_proxy: EventLoopProxy, _native_state: Arc<NativeState>) -> Result<()> {
    Ok(())
}

pub mod icons {
    use crate::icons::ProcessedAsset;

    #[allow(unused_variables)]
    pub fn lookup(name: &str) -> Option<ProcessedAsset> {
        None
    }
}

pub const fn autocomplete_active() -> bool {
    true
}
