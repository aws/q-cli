use std::borrow::Cow;
use std::fmt;

use parking_lot::RwLock;
use tokio::sync::mpsc::UnboundedSender;
use wry::application::dpi::{
    LogicalSize,
    PhysicalPosition,
    PhysicalSize,
    Position,
};
use wry::webview::{
    WebContext,
    WebView,
};

use crate::event::{
    Placement,
    RelativeDirection,
    WindowEvent,
};
use crate::figterm::{
    FigtermCommand,
    FigtermState,
};
use crate::platform::{
    self,
    PlatformState,
};
use crate::utils::Rect;
use crate::AUTOCOMPLETE_ID;

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
    pub context: WebContext,
    pub window_id: WindowId,
    pub anchor: RwLock<PhysicalPosition<i32>>,
    pub position: RwLock<PhysicalPosition<i32>>,
    pub size: RwLock<PhysicalSize<u32>>,
    pub placement: RwLock<Placement>,
}

impl WindowState {
    pub fn new(window_id: WindowId, webview: WebView, context: WebContext) -> Self {
        let position = webview
            .window()
            .inner_position()
            .expect("Failed to acquire window position");

        let size = webview.window().inner_size();

        Self {
            webview,
            context,
            window_id,
            anchor: RwLock::new(PhysicalPosition::default()),
            position: RwLock::new(position),
            size: RwLock::new(size),
            placement: RwLock::new(Placement::Absolute),
        }
    }

    fn update_position(&self, platform_state: &PlatformState) {
        let position = *self.position.read();
        let anchor = *self.anchor.read();
        let size = *self.size.read();
        let placement = *self.placement.read();

        let vertical_padding = anchor.y + 5;

        let x = match placement {
            Placement::Absolute => position.x,
            Placement::RelativeTo((caret, RelativeDirection::Above | RelativeDirection::Below)) => caret.x + anchor.x,
        };

        let y = match placement {
            Placement::Absolute => position.y,
            Placement::RelativeTo((caret, RelativeDirection::Above)) => {
                // todo(mschrage): where do we handle different DPIs? (It should not be here!)
                caret.y - vertical_padding - (size.height * 2) as i32
            },
            Placement::RelativeTo((caret, RelativeDirection::Below)) => caret.max_y() + vertical_padding,
        };

        if let Err(err) = platform_state.position_window(
            self.webview.window(),
            &self.window_id,
            Position::Physical(PhysicalPosition { x, y }),
        ) {
            tracing::error!(%err, window_id =% self.window_id, "Failed to position window");
        }
    }

    pub fn handle(
        &self,
        event: WindowEvent,
        figterm_state: &FigtermState,
        platform_state: &PlatformState,
        api_tx: &UnboundedSender<(WindowId, String)>,
    ) {
        match event {
            WindowEvent::Reanchor { x, y } => {
                *self.anchor.write() = PhysicalPosition { x, y };
                self.update_position(platform_state);
            },
            WindowEvent::PositionAbsolute { x, y } => {
                *self.placement.write() = Placement::Absolute;
                *self.position.write() = PhysicalPosition { x, y };
                self.update_position(platform_state);
            },
            WindowEvent::PositionRelativeToRect {
                x,
                y,
                width,
                height,
                direction,
            } => {
                *self.placement.write() = Placement::RelativeTo((Rect { x, y, width, height }, direction));
                self.update_position(platform_state);
            },
            WindowEvent::Resize { width, height } => {
                *self.size.write() = PhysicalSize { width, height };
                self.update_position(platform_state);
                cfg_if::cfg_if! {
                    if #[cfg(target_os = "linux")] {
                        if self.window_id == AUTOCOMPLETE_ID {
                            self.webview
                                .window()
                                .set_min_inner_size(Some(LogicalSize { width, height }));
                        } else {
                            self.webview.window().set_inner_size(LogicalSize { width, height });
                        }
                    } else {
                        self.webview.window().set_inner_size(LogicalSize { width, height });
                    }
                }
            },
            WindowEvent::Hide => {
                for session in figterm_state.linked_sessions.lock().iter() {
                    let _ = session.sender.send(FigtermCommand::InterceptClear);
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
                for session in figterm_state.linked_sessions.lock().iter() {
                    let _ = session.sender.send(FigtermCommand::InterceptClear);
                }
            },
            WindowEvent::Show => {
                if self.window_id == AUTOCOMPLETE_ID {
                    if platform::autocomplete_active() {
                        self.webview.window().set_visible(true);
                        self.webview.window().set_always_on_top(true);
                        #[cfg(target_os = "windows")]
                        self.webview.window().set_always_on_top(false);
                    }
                } else {
                    self.webview.window().set_visible(true);
                    self.webview.window().set_focus();
                }
            },
            WindowEvent::NavigateAbsolute { url } => {
                self.webview
                    .evaluate_script(&format!("window.location.href = '{url}';"))
                    .unwrap();
            },
            WindowEvent::NavigateRelative { path } => {
                self.webview
                    .evaluate_script(&format!("window.location.pathname = '{path}';"))
                    .unwrap();
            },
            WindowEvent::Emit { event, payload } => {
                self.webview
                    .evaluate_script(&format!(
                        "document.dispatchEvent(new CustomEvent('{event}', {{'detail': `{payload}`}}));"
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
            WindowEvent::DebugMode(debug_mode) => {
                // Macos does not support setting the webview background color so we have
                // to set the css background root color to see the window
                cfg_if::cfg_if! {
                    if #[cfg(target_os = "macos")] {
                        self.webview
                            .evaluate_script(if debug_mode {
                                "document.querySelector(':root').style.setProperty('background-color', 'red');"
                            } else {
                                "document.querySelector(':root').style.removeProperty('background-color');"
                            })
                            .unwrap();
                    } else {
                        self.webview
                            .set_background_color(if debug_mode {
                                (0xff, 0, 0, 0xff)
                            } else {
                                (0, 0, 0, 0) }
                            ).unwrap();
                    }

                }
            },
        }
    }
}
