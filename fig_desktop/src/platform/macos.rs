use std::borrow::Cow;
use std::sync::Arc;

use anyhow::anyhow;
use macos_accessibility_position::caret_position::{
    get_caret_position,
    CaretPosition,
};
use macos_accessibility_position::{
    get_active_window,
    register_observer,
    WindowServer,
    WindowServerEvent,
};
use once_cell::sync::Lazy;
use parking_lot::{
    Mutex,
    RwLock,
};
use tracing::warn;
use wry::application::dpi::Position;

use super::{
    PlatformBoundEvent,
    PlatformWindow,
    WindowGeometry,
};
use crate::event::{
    Event,
    RelativeDirection,
    WindowEvent,
};
use crate::icons::ProcessedAsset;
use crate::utils::Rect;
use crate::webview::window::WindowId;
use crate::{
    EventLoopProxy,
    AUTOCOMPLETE_ID,
};

pub const DEFAULT_CARET_WIDTH: i32 = 10;

static UNMANAGED: Lazy<Unmanaged> = Lazy::new(|| Unmanaged {
    event_sender: RwLock::new(Option::<EventLoopProxy>::None),
    window_server: RwLock::new(Option::<Arc<Mutex<WindowServer>>>::None),
});

struct Unmanaged {
    event_sender: RwLock<Option<EventLoopProxy>>,
    window_server: RwLock<Option<Arc<Mutex<WindowServer>>>>,
}

#[derive(Debug)]
pub struct PlatformStateImpl {
    proxy: EventLoopProxy,
}

impl PlatformStateImpl {
    pub fn new(proxy: EventLoopProxy) -> Self {
        Self { proxy }
    }

    pub fn handle(self: &Arc<Self>, event: PlatformBoundEvent) -> anyhow::Result<()> {
        match event {
            PlatformBoundEvent::Initialize => {
                let observer_proxy = self.proxy.clone();
                UNMANAGED.event_sender.write().replace(self.proxy.clone());
                let (tx, rx) = flume::unbounded::<WindowServerEvent>();
                UNMANAGED
                    .window_server
                    .write()
                    .replace(unsafe { register_observer(tx) });
                tokio::runtime::Handle::current().spawn(async move {
                    while let Ok(result) = rx.recv_async().await {
                        match result {
                            WindowServerEvent::FocusChanged { .. } => {
                                if let Err(e) = observer_proxy.send_event(Event::WindowEvent {
                                    window_id: AUTOCOMPLETE_ID,
                                    window_event: WindowEvent::Hide,
                                }) {
                                    warn!("Error sending event: {e:?}");
                                }
                            },
                        }
                    }
                });
                Ok(())
            },
            PlatformBoundEvent::EditBufferChanged => unsafe {
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
                Err(anyhow!("Failed to acquire caret position"))
            },
        }
    }

    pub(super) fn position_window(
        &self,
        webview_window: &wry::application::window::Window,
        _window_id: &WindowId,
        position: Position,
    ) -> wry::Result<()> {
        webview_window.set_outer_position(position);
        Ok(())
    }

    #[allow(dead_code)]
    pub(super) fn get_cursor_position(&self) -> Option<Rect<i32, i32>> {
        None
    }

    /// Gets the currently active window on the platform
    pub(super) fn get_active_window(&self) -> Option<PlatformWindow> {
        None
    }

    pub(super) fn icon_lookup(_name: &str) -> Option<ProcessedAsset> {
        None
    }

    pub(super) fn shell() -> Cow<'static, str> {
        "/bin/bash".into()
    }

    fn get_window_geometry(&self) -> Option<super::WindowGeometry> {
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
}

pub const fn autocomplete_active() -> bool {
    true
}
