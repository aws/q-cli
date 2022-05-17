use parking_lot::RwLock;
use tokio::sync::mpsc::UnboundedSender;
use tracing::debug;
use wry::application::dpi::{
    PhysicalPosition,
    PhysicalSize,
    Position,
};
use wry::webview::WebView;

use crate::FigId;

#[derive(Debug)]
pub enum FigWindowEvent {
    Reanchor { x: i32, y: i32 },
    Reposition { x: i32, y: i32 },
    UpdateCaret { x: i32, y: i32, width: i32, height: i32 },
    Resize { width: u32, height: u32 },
    Hide,
    Show,
    Emit { event: String, payload: String },
    Api { payload: String },
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

    pub fn handle(&self, event: FigWindowEvent, api_tx: &UnboundedSender<(FigId, String)>) {
        match event {
            FigWindowEvent::Reanchor { x, y } => {
                let position = self.position.read();
                let caret_position = self.caret_position.read();
                *self.anchor.write() = PhysicalPosition { x, y };
                self.webview
                    .window()
                    .set_outer_position(Position::Physical(PhysicalPosition {
                        x: x + position.x + caret_position.x,
                        y: y + position.y + caret_position.y,
                    }))
            },
            FigWindowEvent::Reposition { x, y } => {
                let anchor = self.anchor.read();
                let caret_position = self.caret_position.read();
                *self.position.write() = PhysicalPosition {
                    x: caret_position.x,
                    y: caret_position.y,
                };
                debug!(
                    "x {x} y {y} anchor.x {} anchor.y {} caret_position.x {} caret_position.y {}",
                    anchor.x, anchor.y, caret_position.x, caret_position.y
                );
                self.webview
                    .window()
                    .set_outer_position(Position::Physical(PhysicalPosition {
                        x: caret_position.x,
                        y: caret_position.y,
                    }))
            },
            FigWindowEvent::UpdateCaret { x, y, width, height } => {
                let anchor = PhysicalPosition { x, y };
                let position = self.position.read();
                *self.caret_position.write() = PhysicalPosition { x, y };
                *self.caret_size.write() = PhysicalSize { width, height };
                self.webview
                    .window()
                    .set_outer_position(Position::Physical(PhysicalPosition { x, y }))
            },
            FigWindowEvent::Resize { width, height } => {
                self.webview.window().set_inner_size(PhysicalSize { width, height })
            },
            FigWindowEvent::Hide => self.webview.window().set_visible(false),
            FigWindowEvent::Show => {
                self.webview.window().set_visible(true);
                self.webview.window().set_always_on_top(true);
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
        }
    }
}
