use wry::application::event_loop::ControlFlow;

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
    PlatformBoundEvent(PlatformBoundEvent),

    ControlFlow(ControlFlow),

    RefreshDebugger,
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

impl<T> Rect<T, T>
where
    T: std::ops::Add<Output = T> + Copy,
{
    #[allow(dead_code)]
    pub fn max_x(&self) -> T {
        self.x + self.width
    }

    pub fn max_y(&self) -> T {
        self.y + self.height
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Placement {
    Absolute,
    RelativeTo((Rect<i32, i32>, RelativeDirection)),
}

#[derive(Debug)]
pub enum WindowEvent {
    Reanchor {
        x: i32,
        y: i32,
    },
    PositionRelativeToRect {
        /// x position of cursor
        x: i32,
        /// y position of cursor
        y: i32,
        /// width of cursor
        width: i32,
        /// height of cursor
        height: i32,
        direction: RelativeDirection,
    },
    PositionAbsolute {
        x: i32,
        y: i32,
    },
    Resize {
        width: u32,
        height: u32,
    },
    /// Hides the window
    Hide,
    /// Request to hide the window, may not be respected
    HideSoft,
    Show,
    Emit {
        event: String,
        payload: String,
    },
    NatigateRelative {
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
}
