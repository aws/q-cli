use wry::application::dpi::{
    LogicalPosition,
    LogicalSize,
};
use wry::application::event_loop::ControlFlow;
use wry::application::window::Theme;

use crate::platform::PlatformBoundEvent;
use crate::utils::Rect;
use crate::webview::window::WindowId;

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum Event {
    WindowEvent {
        window_id: WindowId,
        window_event: WindowEvent,
    },
    WindowEventAll {
        window_event: WindowEvent,
    },

    PlatformBoundEvent(PlatformBoundEvent),
    ControlFlow(ControlFlow),
    SetTrayEnabled(bool),

    ReloadCredentials,
    ReloadAccessibility,
    ReloadTray,
}

impl From<PlatformBoundEvent> for Event {
    fn from(event: PlatformBoundEvent) -> Self {
        Self::PlatformBoundEvent(event)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RelativeDirection {
    Above,
    Below,
}

#[derive(Debug, Clone, Copy)]
pub enum ClippingBehavior {
    // Allow window to be clipped
    Allow,
    // Offset window position to keep it in screen frame
    KeepInFrame,
}

#[derive(Debug, Clone, Copy)]
pub enum Placement {
    Absolute,
    RelativeTo(Rect, RelativeDirection, ClippingBehavior),
}

#[derive(Debug, Clone)]
pub enum EmitEventName {
    Notification,
    ProtoMessageReceived,
    GlobalErrorOccurred,
}

impl std::fmt::Display for EmitEventName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Notification | Self::ProtoMessageReceived => "FigProtoMessageRecieved",
            Self::GlobalErrorOccurred => "FigGlobalErrorOccurred",
        })
    }
}

#[derive(Debug, Clone)]
pub enum WindowEvent {
    /// Sets the window to be enabled or disabled
    ///
    /// This will cause events to be ignored other than [`WindowEvent::Hide`] and
    /// [`WindowEvent::SetEnabled(true)`]
    SetEnabled(bool),
    /// Sets the theme of the window (light, dark, or system if None)
    ///
    /// This is currently unimplemented blocked on https://github.com/tauri-apps/tao/issues/582
    SetTheme(Option<Theme>),
    Reanchor {
        position: LogicalPosition<f64>,
    },
    PositionRelativeToCaret {
        caret: Rect,
    },
    // todo(mschrage): move direction and clipping behavior out of this struct into WindowState
    PositionRelativeToRect {
        rect: Rect,
        direction: RelativeDirection,
        /// Defines behavior when desired window position is outside of screen
        clipping_behavior: ClippingBehavior,
    },
    PositionAbsolute {
        position: LogicalPosition<f64>,
    },
    Resize {
        size: LogicalSize<f64>,
    },
    /// Hides the window
    Hide,
    Show,
    Emit {
        event_name: EmitEventName,
        payload: String,
    },
    NavigateRelative {
        path: String,
    },
    NavigateAbsolute {
        url: url::Url,
    },
    Api {
        /// A base64 encoded protobuf
        payload: String,
    },
    Devtools,
    DebugMode(bool),
    Center,
}

impl WindowEvent {
    pub fn is_allowed_while_disabled(&self) -> bool {
        matches!(
            self,
            WindowEvent::Hide
                | WindowEvent::SetEnabled(_)
                // TODO: we really shouldnt need to allow these to be called when disabled, 
                // however we allow them at the moment because notification listeners are
                // initialized early on and we dont have a way to delay them until the window
                // is enabled
                | WindowEvent::Api { .. }
                | WindowEvent::Emit {
                    event_name: EmitEventName::GlobalErrorOccurred | EmitEventName::ProtoMessageReceived,
                    ..
                }
        )
    }
}
