use std::borrow::Cow;
use std::sync::Arc;

use wry::application::dpi::Position;

use crate::icons::ProcessedAsset;
use crate::utils::Rect;
use crate::webview::window::WindowId;
use crate::EventLoopProxy;

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

#[allow(dead_code)]
pub type WindowGeometry = Rect<i32, i32>;

pub struct PlatformWindow {
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
    pub fn handle(self: &Arc<Self>, event: PlatformBoundEvent) -> anyhow::Result<()> {
        self.clone().0.handle(event)
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
    pub fn get_cursor_position(&self) -> Option<Rect<i32, i32>> {
        self.0.get_cursor_position()
    }

    /// Gets the currently active window on the platform
    pub fn get_active_window(&self) -> Option<PlatformWindow> {
        self.0.get_active_window()
    }

    /// Looks up icons by name on the platform
    pub fn icon_lookup(name: &str) -> Option<ProcessedAsset> {
        PlatformStateImpl::icon_lookup(name)
    }

    /// The shell to execute processes in
    pub fn shell() -> Cow<'static, str> {
        PlatformStateImpl::shell()
    }
}

#[derive(Debug)]
pub enum PlatformBoundEvent {
    Initialize,
    EditBufferChanged,
}
