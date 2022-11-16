use std::borrow::Cow;

use wry::application::dpi::{
    LogicalSize,
    Position,
    Size,
};
use wry::application::event_loop::ControlFlow;
use wry::application::window::Theme;

use crate::platform::PlatformBoundEvent;
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

    ShowMessageNotification {
        title: Cow<'static, str>,
        body: Cow<'static, str>,
        parent: Option<WindowId>,
    },
}

impl From<PlatformBoundEvent> for Event {
    fn from(event: PlatformBoundEvent) -> Self {
        Self::PlatformBoundEvent(event)
    }
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowPosition {
    Absolute(Position),
    Centered,
    RelativeToCaret { caret_position: Position, caret_size: Size },
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
    UpdateWindowGeometry {
        position: Option<WindowPosition>,
        size: Option<LogicalSize<f64>>,
        anchor: Option<LogicalSize<f64>>,
    },
    /// Hides the window
    Hide,
    Show,
    Emit {
        event_name: EmitEventName,
        payload: Cow<'static, str>,
    },
    NavigateRelative {
        path: Cow<'static, str>,
    },
    NavigateAbsolute {
        url: url::Url,
    },
    Reload,

    /// Trigger a reload if the page is not already loaded
    ReloadIfNotLoaded,

    Api {
        /// A base64 encoded protobuf
        payload: Cow<'static, str>,
    },
    Devtools,
    DebugMode(bool),

    SetHtml {
        html: Cow<'static, str>,
    },

    Batch(Vec<WindowEvent>),
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
