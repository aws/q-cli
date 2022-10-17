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
        event: WindowEvent,
    },

    PlatformBoundEvent(PlatformBoundEvent),
    ControlFlow(ControlFlow),
    ReloadTray,
    SetTrayEnabled(bool),
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
    RelativeTo((Rect, RelativeDirection, ClippingBehavior)),
}

#[derive(Debug, Clone)]
pub enum EmitEventName {
    ProtoMessageReceived,
    GlobalErrorOccurred,
}

impl std::fmt::Display for EmitEventName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::ProtoMessageReceived => "FigProtoMessageRecieved",
            Self::GlobalErrorOccurred => "FigGlobalErrorOccurred",
        })
    }
}

#[derive(Debug, Clone)]
pub enum WindowEvent {
    SetEnabled(bool),
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
        // Defines behavior when desired window position is outside of screen
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
        matches!(self, WindowEvent::Hide)
    }
}
