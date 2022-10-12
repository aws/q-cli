use std::borrow::Cow;
use std::sync::Arc;

use anyhow::{
    anyhow,
    Ok,
};
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
use wry::application::dpi::{
    LogicalPosition,
    LogicalSize,
    Position,
};
use wry::application::platform::macos::{
    ActivationPolicy,
    EventLoopWindowTargetExtMacOS,
};

use super::{
    PlatformBoundEvent,
    PlatformWindow,
    WindowGeometry,
};
use crate::event::{
    ClippingBehavior,
    Event,
    RelativeDirection,
    WindowEvent,
};
use crate::icons::ProcessedAsset;
use crate::utils::Rect;
use crate::webview::window::WindowId;
use crate::webview::FigWindowMap;
use crate::{
    EventLoopProxy,
    EventLoopWindowTarget,
    AUTOCOMPLETE_ID,
    MISSION_CONTROL_ID,
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

    pub fn handle(
        self: &Arc<Self>,
        event: PlatformBoundEvent,
        window_target: &EventLoopWindowTarget,
        window_map: &FigWindowMap,
    ) -> anyhow::Result<()> {
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
                    let update_fullscreen = |fullscreen: bool| {
                        Event::PlatformBoundEvent(PlatformBoundEvent::FullscreenStateUpdated { fullscreen })
                    };
                    while let std::result::Result::Ok(result) = rx.recv_async().await {
                        match result {
                            WindowServerEvent::FocusChanged { window, .. } => {
                                if let Err(e) = observer_proxy.send_event(Event::WindowEvent {
                                    window_id: AUTOCOMPLETE_ID,
                                    window_event: WindowEvent::Hide,
                                }) {
                                    warn!("Error sending event: {e:?}");
                                }
                                if let Some(window) = window {
                                    let is_fullscreen = unsafe { window.is_fullscreen().unwrap_or(false) };
                                    if let Err(e) = observer_proxy.send_event(update_fullscreen(is_fullscreen)) {
                                        warn!("Error sending event: {e:?}");
                                    }
                                }
                            },
                            WindowServerEvent::ActiveSpaceChanged { is_fullscreen } => {
                                if let Err(e) = observer_proxy.send_event(update_fullscreen(is_fullscreen)) {
                                    warn!("Error sending event: {e:?}");
                                }
                            },
                        }
                    }
                });
                Ok(())
            },
            PlatformBoundEvent::EditBufferChanged => {
                // todo(mschrage): move all positioning logic into cross platform `windows.rs` file
                // This event should only update the position of the "relative rect"
                let caret = match self.get_cursor_position() {
                    Some(frame) => frame,
                    None => return Err(anyhow!("Failed to acquire caret position")),
                };

                let monitor_frame = match window_map.get(&AUTOCOMPLETE_ID) {
                    Some(window) => match self.get_current_monitor_frame(window.webview.window()) {
                        Some(frame) => frame,
                        None => return Err(anyhow!("Failed to acquire monitor frame")),
                    },
                    None => return Err(anyhow!("Failed to acquire autocomplete window reference")),
                };

                let window_frame = match self.get_window_geometry() {
                    Some(frame) => frame,
                    None => return Err(anyhow!("Failed to acquire current window frame")),
                };

                // Caret origin will always be less than window origin (if coordinate system origin is top-left)
                assert!(caret.y >= window_frame.y);

                let max_height = fig_settings::settings::get_int_or("autocomplete.height", 140) as i32;

                // TODO: this calculation does not take into account anchor offset (or default vertical padding)
                let is_above = window_frame.max_y() < caret.max_y() + max_height && // If positioned below, will popup appear inside of window frame?
                                          monitor_frame.y < caret.y - max_height; // If positioned above, will autocomplete go outside of bounds of current monitor?

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
                            x: caret.x,
                            y: caret.y,
                            width: caret.width,
                            height: caret.height,
                            direction,
                            clipping_behavior: ClippingBehavior::KeepInFrame,
                        },
                    })
                    .ok();

                Ok(())
            },
            PlatformBoundEvent::FullscreenStateUpdated { fullscreen } => {
                let policy = if fullscreen {
                    ActivationPolicy::Accessory
                } else {
                    let mission_control_visible = window_map
                        .get(&MISSION_CONTROL_ID)
                        .map(|window| window.webview.window().is_visible())
                        .unwrap_or(false);

                    if mission_control_visible {
                        ActivationPolicy::Regular
                    } else {
                        ActivationPolicy::Accessory
                    }
                };
                window_target.set_activation_policy_at_runtime(policy);
                Ok(())
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
        std::result::Result::Ok(())
    }

    pub(super) fn get_cursor_position(&self) -> Option<Rect<i32, i32>> {
        let caret: CaretPosition = unsafe { get_caret_position(true) };

        if caret.valid {
            Some(Rect {
                x: caret.x as i32,
                y: caret.y as i32,
                width: DEFAULT_CARET_WIDTH,
                height: caret.height as i32,
            })
        } else {
            None
        }
    }

    pub fn get_current_monitor_frame(&self, window: &wry::application::window::Window) -> Option<Rect<i32, i32>> {
        match window.current_monitor() {
            Some(monitor) => {
                let origin = monitor.position().to_logical(monitor.scale_factor()) as LogicalPosition<i32>;
                let size = monitor.size().to_logical(monitor.scale_factor()) as LogicalSize<i32>;

                Some(Rect {
                    x: origin.x,
                    y: origin.y,
                    width: size.width as i32,
                    height: size.height as i32,
                })
            },
            None => None,
        }
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
                width: window.position.width as i32,
                height: window.position.height as i32,
            }),
            None => None,
        }
    }
}

pub const fn autocomplete_active() -> bool {
    true
}
