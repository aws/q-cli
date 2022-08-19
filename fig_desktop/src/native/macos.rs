use std::sync::Arc;

use anyhow::Result;

use crate::event::NativeEvent;
use crate::webview::window::WindowId;
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

    pub fn get_window_geometry(&self) -> Option<super::WindowGeometry> {
        None
    }

    pub fn position_window(&self, _window_id: &WindowId, _x: i32, _y: i32, fallback: impl FnOnce()) {
        fallback();
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
