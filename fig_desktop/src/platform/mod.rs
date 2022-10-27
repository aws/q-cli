use std::borrow::Cow;
use std::sync::Arc;

use wry::application::dpi::Position;

use crate::icons::{
    AssetSpecifier,
    ProcessedAsset,
};
use crate::utils::Rect;
use crate::webview::window::WindowId;
use crate::webview::FigIdMap;
use crate::{
    EventLoopProxy,
    EventLoopWindowTarget,
};

cfg_if::cfg_if! {
    if #[cfg(target_os="linux")] {
        mod linux;
        pub use self::linux::*;
    } else if #[cfg(target_os="macos")] {
        mod macos;
        pub use self::macos::*;
    } else if #[cfg(windows)] {
        mod windows;
        pub use self::windows::*;
    } else {
        compile_error!("Unsupported platform");
    }
}

pub struct PlatformWindow {
    pub rect: Rect,
    pub inner: PlatformWindowImpl,
    // TODO: add a platform specific impl of things like name, is_terminal(), etc
    // pub inner: ExternalPlatformWindowImpl
}

pub struct PlatformState(Arc<PlatformStateImpl>);

impl PlatformState {
    /// Create a new PlatformState instance
    pub fn new(proxy: EventLoopProxy) -> Self {
        Self(Arc::new(PlatformStateImpl::new(proxy)))
    }

    /// Handle a [`PlatformBoundEvent`]
    pub fn handle(
        self: &Arc<Self>,
        event: PlatformBoundEvent,
        window_target: &EventLoopWindowTarget,
        window_map: &FigIdMap,
    ) -> anyhow::Result<()> {
        self.clone().0.handle(event, window_target, window_map)
    }

    /// Position the window at the given coordinates
    pub fn position_window(
        &self,
        webview_window: &wry::application::window::Window,
        window_id: &WindowId,
        position: Position,
    ) -> wry::Result<()> {
        self.0.position_window(webview_window, window_id, position)
    }

    /// Gets the current cursor position on the screen
    #[allow(dead_code)]
    pub fn get_cursor_position(&self) -> Option<Rect> {
        self.0.get_cursor_position()
    }

    pub fn get_current_monitor_frame(&self, window: &wry::application::window::Window) -> Option<Rect> {
        let cursor_position = match self.get_cursor_position() {
            Some(cursor_position) => cursor_position.position,
            None => return None,
        };

        window
            .available_monitors()
            .map(|monitor| {
                let scale_factor = monitor.scale_factor();
                Rect {
                    position: monitor.position().to_logical(scale_factor),
                    size: monitor.size().to_logical(scale_factor),
                }
            })
            .find(|bounds| bounds.contains(cursor_position))
    }

    /// Gets the currently active window on the platform
    pub fn get_active_window(&self) -> Option<PlatformWindow> {
        self.0.get_active_window()
    }

    /// Looks up icons by name on the platform
    pub fn icon_lookup(name: &AssetSpecifier) -> Option<ProcessedAsset> {
        PlatformStateImpl::icon_lookup(name)
    }

    /// The shell to execute processes in
    pub fn shell() -> Cow<'static, str> {
        PlatformStateImpl::shell()
    }

    /// Whether or not accessibility is enabled
    pub fn accessibility_is_enabled() -> Option<bool> {
        PlatformStateImpl::accessibility_is_enabled()
    }
}

#[derive(Debug)]
pub enum PlatformBoundEvent {
    Initialize,
    EditBufferChanged,
    FullscreenStateUpdated {
        fullscreen: bool,
    },
    AccessibilityUpdated {
        enabled: bool,
    },
    AppWindowFocusChanged {
        window_id: WindowId,
        focused: bool,
        fullscreen: bool,
    },
    CaretPositionUpdateRequested,
    WindowDestroyed {
        window: PlatformWindowImpl,
    },
    ExternalWindowFocusChanged {
        window: PlatformWindowImpl,
    },
}
