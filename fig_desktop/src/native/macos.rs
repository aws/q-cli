use std::sync::Arc;

use anyhow::{
    anyhow,
    Result,
};
use macos_accessibility_position::mac::caret::caret_position::CaretPosition;
use macos_accessibility_position::mac::caret::get_caret_position;
use macos_accessibility_position::{
    get_active_window,
    register_observer,
};
use once_cell::sync::Lazy;
use parking_lot::RwLock;

use super::WindowGeometry;
use crate::event::{
    Event,
    NativeEvent,
    RelativeDirection,
    WindowEvent,
};
use crate::webview::window::WindowId;
use crate::{
    EventLoopProxy,
    AUTOCOMPLETE_ID,
};

pub const DEFAULT_CARET_WIDTH: i32 = 10;
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
                let is_above = match self.get_window_geometry() {
                    Some(window_frame) => {
                        window_frame.y + window_frame.height
                            < (caret_position.y as i32)
                                + (caret_position.height as i32)
                                + fig_settings::settings::get_int_or("autocomplete.height", 140) as i32
                    },
                    None => false,
                };

                if caret_position.valid {
                    let x = (caret_position.x * 2.0) as i32;
                    let y = (caret_position.y * 2.0) as i32;
                    let height = caret_position.height as i32 * 2;

                    let direction = match is_above {
                        true => RelativeDirection::Above,
                        false => RelativeDirection::Below,
                    };
                    UNMANAGED
                        .event_sender
                        .read()
                        .clone()
                        .unwrap()
                        .send_event(Event::WindowEvent {
                            window_id: AUTOCOMPLETE_ID,
                            window_event: WindowEvent::PositionRelativeToRect {
                                x,
                                y,
                                width: DEFAULT_CARET_WIDTH,
                                height,
                                direction,
                            },
                        })
                        .ok();
                }
            },
        }
        Err(anyhow!("Failed to acquire caret position"))
    }

    pub fn get_window_geometry(&self) -> Option<super::WindowGeometry> {
        match get_active_window() {
            Some(window) => Some(WindowGeometry {
                x: window.position.x as i32,
                y: window.position.y as i32,
                width: window.position.height as i32,
                height: window.position.height as i32,
            }),
            None => None,
        }
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
