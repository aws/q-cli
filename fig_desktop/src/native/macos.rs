use std::sync::Arc;

use anyhow::{
    anyhow,
    Result,
};
use macos_accessibility_position::mac::caret::caret_position::CaretPosition;
use macos_accessibility_position::mac::caret::get_caret_position;
use macos_accessibility_position::register_observer;
use once_cell::sync::Lazy;
use parking_lot::RwLock;

use crate::event::{
    Event,
    NativeEvent,
    WindowEvent,
};
use crate::webview::window::WindowId;
use crate::{
    EventLoopProxy,
    AUTOCOMPLETE_ID,
};

pub const SHELL: &str = "/bin/bash";

#[derive(Debug, Default)]
pub struct NativeState;

impl NativeState {
    pub fn new(_proxy: EventLoopProxy) -> Self {
        Self {}
    }

    pub fn handle(&self, event: NativeEvent) -> Result<()> {
        match event {
            NativeEvent::EditBufferChanged => unsafe {
                let caret_position: CaretPosition = get_caret_position(true);
                if caret_position.valid {
                    UNMANAGED
                        .event_sender
                        .read()
                        .clone()
                        .unwrap()
                        .send_event(Event::WindowEvent {
                            window_id: AUTOCOMPLETE_ID,
                            window_event: WindowEvent::Reposition {
                                x: ((caret_position.x + 5.0) * 2.0) as i32,
                                y: ((caret_position.y + caret_position.height) * 2.0) as i32,
                            },
                        })
                        .ok();
                }
            },
        }
        Err(anyhow!("Failed to acquire caret position"))
    }

    pub fn get_window_geometry(&self) -> Option<super::WindowGeometry> {
        None
    }

    pub fn position_window(&self, _window_id: &WindowId, _x: i32, _y: i32, fallback: impl FnOnce()) {
        fallback();
    }
}

static UNMANAGED: Lazy<Unmanaged> = Lazy::new(|| Unmanaged {
    event_sender: RwLock::new(Option::<EventLoopProxy>::None),
});

struct Unmanaged {
    event_sender: RwLock<Option<EventLoopProxy>>,
}

pub async fn init(proxy: EventLoopProxy, _native_state: Arc<NativeState>) -> Result<()> {
    UNMANAGED.event_sender.write().replace(proxy);
    // tokio::spawn(async { handle_macos().await });
    unsafe {
        register_observer();
    }
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
