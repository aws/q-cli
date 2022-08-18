use std::borrow::Cow;
use std::fmt;

use parking_lot::RwLock;
use tokio::runtime::Handle;
use tokio::sync::mpsc::UnboundedSender;
use wry::application::dpi::{
    LogicalSize,
    PhysicalPosition,
    PhysicalSize,
    Position,
};
use wry::webview::WebView;

use crate::event::WindowEvent;
use crate::figterm::{
    FigtermCommand,
    FigtermState,
};
use crate::native;

#[allow(unused)]
pub enum CursorPositionKind {
    Absolute,
    Relative,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WindowId(pub Cow<'static, str>);

impl fmt::Display for WindowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// TODO: Add state for the active terminal window
pub struct WindowState {
    pub webview: WebView,
    pub window_id: WindowId,
    pub anchor: RwLock<PhysicalPosition<i32>>,
    pub position: RwLock<PhysicalPosition<i32>>,
}

impl WindowState {
    pub fn new(window_id: WindowId, webview: WebView) -> Self {
        let position = webview
            .window()
            .inner_position()
            .expect("Failed to acquire window position");

        Self {
            window_id,
            webview,
            anchor: RwLock::new(PhysicalPosition::default()),
            position: RwLock::new(position),
        }
    }

    fn update_position(&self) {
        let positon = *self.position.read();
        let anchor = *self.anchor.read();
        self.webview
            .window()
            .set_outer_position(Position::Physical(PhysicalPosition {
                x: positon.x + anchor.x,
                y: positon.y + anchor.y,
            }));
    }

    pub fn handle(
        &self,
        event: WindowEvent,
        figterm_state: &FigtermState,
        api_tx: &UnboundedSender<(WindowId, String)>,
    ) {
        match event {
            WindowEvent::Reanchor { x, y } => {
                *self.anchor.write() = PhysicalPosition { x, y };
                self.update_position();
            },
            WindowEvent::Reposition { x, y } => {
                *self.position.write() = PhysicalPosition { x, y };
                self.update_position();
            },
            WindowEvent::Resize { width, height } => {
                #[cfg(target_os = "linux")]
                self.webview
                    .window()
                    .set_min_inner_size(Some(LogicalSize { width, height }));
                #[cfg(not(target_os = "linux"))]
                self.webview.window().set_inner_size(LogicalSize { width, height });
            },
            WindowEvent::Hide => {
                for session in figterm_state.sessions.iter() {
                    let sender = session.sender.clone();
                    Handle::current().spawn(async move {
                        let _ = sender.send(FigtermCommand::InterceptClear);
                    });
                }
                self.webview.window().set_visible(false);
                #[cfg(not(target_os = "linux"))]
                self.webview.window().set_resizable(true);
                #[cfg(target_os = "linux")]
                self.webview
                    .window()
                    .set_min_inner_size(Some(PhysicalSize { width: 1, height: 1 }));
                self.webview
                    .window()
                    .set_inner_size(PhysicalSize { width: 1, height: 1 });
                #[cfg(not(target_os = "linux"))]
                self.webview.window().set_resizable(false);
            },
            WindowEvent::HideSoft => {
                for session in figterm_state.sessions.iter() {
                    let sender = session.sender.clone();
                    Handle::current().spawn(async move {
                        let _ = sender.send(FigtermCommand::InterceptClear);
                    });
                }
            },
            WindowEvent::Show => {
                if native::autocomplete_active() {
                    self.webview.window().set_visible(true);
                    self.webview.window().set_always_on_top(true);
                }
            },
            WindowEvent::Navigate { url } => {
                self.webview
                    .evaluate_script(&format!("window.location.href = '{url}'"))
                    .unwrap();
            },
            WindowEvent::Emit { event, payload } => {
                self.webview
                    .evaluate_script(&format!(
                        "document.dispatchEvent(new CustomEvent('{event}', {{'detail': `{payload}`}}))"
                    ))
                    .unwrap();
            },
            WindowEvent::Api { payload } => {
                api_tx.send((self.window_id.clone(), payload)).unwrap();
            },
            WindowEvent::Devtools => {
                if self.webview.is_devtools_open() {
                    self.webview.close_devtools();
                } else {
                    self.webview.open_devtools();
                }
            },
        }
    }
}
