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

use crate::figterm::FigTermCommand;
use crate::{
    native,
    FigId,
    GlobalState,
};

#[allow(unused)]
pub enum CursorPositionKind {
    Absolute,
    Relative,
}

#[derive(Debug)]
pub enum FigWindowEvent {
    Reanchor { x: i32, y: i32 },
    Reposition { x: i32, y: i32 },
    UpdateCaret { x: i32, y: i32, width: i32, height: i32 },
    Resize { width: u32, height: u32 },
    Hide,
    Show,
    Emit { event: String, payload: String },
    Navigate { url: url::Url },
    Api { payload: String },
    Devtools,
}

pub struct WindowState {
    pub webview: WebView,
    pub fig_id: FigId,
    pub anchor: RwLock<PhysicalPosition<i32>>,
    pub position: RwLock<PhysicalPosition<i32>>,
    pub caret_position: RwLock<PhysicalPosition<i32>>,
    pub caret_size: RwLock<PhysicalSize<i32>>,
}

impl WindowState {
    pub fn new(fig_id: FigId, webview: WebView) -> Self {
        let position = webview
            .window()
            .inner_position()
            .expect("Failed to acquire window position");

        Self {
            fig_id,
            webview,
            anchor: RwLock::new(PhysicalPosition::default()),
            position: RwLock::new(position),
            caret_position: RwLock::new(PhysicalPosition::default()),
            caret_size: RwLock::new(PhysicalSize::default()),
        }
    }

    pub fn handle(&self, event: FigWindowEvent, state: &GlobalState, api_tx: &UnboundedSender<(FigId, String)>) {
        match event {
            FigWindowEvent::Reanchor { x, y } => {
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
            FigWindowEvent::Reposition { x, y } => {
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
            FigWindowEvent::UpdateCaret { x, y, width, height } => {
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
            FigWindowEvent::Resize { width, height } => self
                .webview
                .window()
                .set_min_inner_size(Some(PhysicalSize { width, height })),
            FigWindowEvent::Hide => {
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
            FigWindowEvent::Show => {
                self.webview.window().set_visible(true);
                self.webview.window().set_always_on_top(true);
            },
            FigWindowEvent::Navigate { url } => {
                self.webview
                    .evaluate_script(&format!("window.location.href = '{url}'"))
                    .unwrap();
            },
            FigWindowEvent::Emit { event, payload } => {
                self.webview
                    .evaluate_script(&format!(
                        "document.dispatchEvent(new CustomEvent('{event}', {{'detail': `{payload}`}}))"
                    ))
                    .unwrap();
                self.webview
                    .evaluate_script(&format!("console.log('Executing {event}')"))
                    .unwrap();
            },
            FigWindowEvent::Api { payload } => {
                api_tx.send((self.fig_id.clone(), payload)).unwrap();
            },
            FigWindowEvent::Devtools => {
                if self.webview.is_devtools_open() {
                    self.webview.close_devtools();
                } else {
                    self.webview.open_devtools();
                }
            },
        }
    }
}
