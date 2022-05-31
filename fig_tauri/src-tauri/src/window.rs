use std::borrow::Cow;
use std::fmt;

use parking_lot::RwLock;
use tokio::runtime::Handle;
use tokio::sync::mpsc::UnboundedSender;
use wry::application::dpi::{
    PhysicalPosition,
    PhysicalSize,
    Position,
    Size,
};
use wry::webview::WebView;

use crate::event::WindowEvent;
use crate::figterm::FigTermCommand;
use crate::{
    native,
    GlobalState,
};

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

pub struct WindowState {
    pub webview: WebView,
    pub window_id: WindowId,
    pub anchor: RwLock<PhysicalPosition<i32>>,
    pub position: RwLock<PhysicalPosition<i32>>,
    pub caret_position: RwLock<PhysicalPosition<i32>>,
    pub caret_size: RwLock<PhysicalSize<i32>>,
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
            caret_position: RwLock::new(PhysicalPosition::default()),
            caret_size: RwLock::new(PhysicalSize::default()),
        }
    }

    pub fn handle(&self, event: WindowEvent, state: &GlobalState, api_tx: &UnboundedSender<(WindowId, String)>) {
        match event {
            WindowEvent::Reanchor { x, y } => {
                let position = self.position.read();
                let caret_position = self.caret_position.read();
                let caret_size = self.caret_size.read();
                *self.anchor.write() = PhysicalPosition { x, y };
                match native::CURSOR_POSITION_KIND {
                    CursorPositionKind::Absolute => {
                        self.webview
                            .window()
                            .set_outer_position(Position::Physical(PhysicalPosition {
                                x: caret_position.x + position.x,
                                y: caret_position.y + position.y + caret_size.height,
                            }))
                    },
                    CursorPositionKind::Relative => {
                        self.webview
                            .window()
                            .set_outer_position(Position::Physical(PhysicalPosition {
                                x: x + caret_position.x + position.x,
                                y: y + caret_position.y + position.y + caret_size.height,
                            }))
                    },
                }
            },
            WindowEvent::Reposition { x, y } => {
                let caret_position = self.caret_position.read();
                let caret_size = self.caret_size.read();
                *self.position.write() = PhysicalPosition {
                    x: caret_position.x,
                    y: caret_position.y,
                };
                match native::CURSOR_POSITION_KIND {
                    CursorPositionKind::Absolute => {
                        self.webview
                            .window()
                            .set_outer_position(Position::Physical(PhysicalPosition {
                                x: x + caret_position.x,
                                y: y + caret_position.y + caret_size.height,
                            }))
                    },
                    CursorPositionKind::Relative => {
                        let anchor = self.anchor.read();
                        self.webview
                            .window()
                            .set_outer_position(Position::Physical(PhysicalPosition {
                                x: anchor.x + caret_position.x + x,
                                y: anchor.y + caret_position.y + y + caret_size.height,
                            }))
                    },
                }
            },
            WindowEvent::UpdateCaret { x, y, width, height } => {
                let position = self.position.read();
                *self.caret_position.write() = PhysicalPosition { x, y };
                *self.caret_size.write() = PhysicalSize { width, height };
                if x == 0 && y == 0 {
                    self.webview.window().set_visible(false);
                }
                match native::CURSOR_POSITION_KIND {
                    CursorPositionKind::Absolute => {
                        self.webview
                            .window()
                            .set_outer_position(Position::Physical(PhysicalPosition {
                                x: x + position.x,
                                y: y + position.y + height,
                            }))
                    },
                    CursorPositionKind::Relative => {
                        let anchor = PhysicalPosition { x, y };
                        self.webview
                            .window()
                            .set_outer_position(Position::Physical(PhysicalPosition {
                                x: anchor.x + x + position.x,
                                y: anchor.y + y + position.y + height,
                            }))
                    },
                }
            },
            WindowEvent::Resize { width, height } => self
                .webview
                .window()
                .set_min_inner_size(Some(PhysicalSize { width, height })),
            WindowEvent::Hide => {
                if let Some(session) = state.figterm_state.most_recent_session() {
                    Handle::current().spawn(async move {
                        session.sender.send(FigTermCommand::ClearIntercept).await.unwrap();
                    });
                }
                self.webview.window().set_visible(false);
                self.webview
                    .window()
                    .set_min_inner_size(Some(Size::Physical(PhysicalSize { width: 1, height: 1 })));
                self.webview
                    .window()
                    .set_inner_size(Size::Physical(PhysicalSize { width: 1, height: 1 }));
            },
            WindowEvent::Show => {
                self.webview.window().set_visible(true);
                self.webview.window().set_always_on_top(true);
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
