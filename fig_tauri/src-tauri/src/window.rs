use std::sync::Arc;

use parking_lot::RwLock;
use tauri::{
    PhysicalPosition,
    PhysicalSize,
    Position,
    Size,
    Window,
};
use tokio::sync::mpsc::{
    UnboundedReceiver,
    UnboundedSender,
};

#[derive(Debug)]
pub enum WindowEvent {
    Reanchor { x: i32, y: i32 },
    Reposition { x: i32, y: i32 },
    UpdateCaret { x: i32, y: i32, width: i32, height: i32 },
    Resize { width: u32, height: u32 },
    Hide,
    Show,
    Emit { event: &'static str, payload: String },
}

#[derive(Debug)]
pub struct WindowState {
    event_sender: RwLock<UnboundedSender<WindowEvent>>,
    anchor: RwLock<PhysicalPosition<i32>>,
    position: RwLock<PhysicalPosition<i32>>,
    caret_position: RwLock<PhysicalPosition<i32>>,
    caret_size: RwLock<PhysicalSize<i32>>,
}

impl WindowState {
    pub fn new(window: &Window, event_sender: UnboundedSender<WindowEvent>) -> Self {
        Self {
            event_sender: RwLock::new(event_sender),
            anchor: RwLock::new(PhysicalPosition::default()),
            position: RwLock::new(window.inner_position().expect("Failed to acquire window position")),
            caret_position: RwLock::new(PhysicalPosition::default()),
            caret_size: RwLock::new(PhysicalSize::default()),
        }
    }

    pub fn send_event(&self, event: WindowEvent) {
        self.event_sender
            .read()
            .send(event)
            .expect("Failed to send window event");
    }
}

pub async fn handle_window(window: Window, mut recv: UnboundedReceiver<WindowEvent>, state: Arc<WindowState>) {
    while let Some(event) = recv.recv().await {
        match event {
            WindowEvent::Reanchor { x, y } => {
                let position = state.position.read();
                let caret_position = state.caret_position.read();
                *state.anchor.write() = PhysicalPosition { x, y };
                window.set_position(Position::Physical(PhysicalPosition {
                    x: x + position.x + caret_position.x,
                    y: y + position.y + caret_position.y,
                }))
            },
            WindowEvent::Reposition { x, y } => {
                let anchor = state.anchor.read();
                let caret_position = state.caret_position.read();
                *state.position.write() = PhysicalPosition { x, y };
                window.set_position(Position::Physical(PhysicalPosition {
                    x: anchor.x + x + caret_position.x,
                    y: anchor.y + y + caret_position.y,
                }))
            },
            WindowEvent::UpdateCaret { x, y, width, height } => {
                let anchor = PhysicalPosition { x, y };
                let position = state.position.read();
                *state.caret_position.write() = PhysicalPosition { x, y };
                *state.caret_size.write() = PhysicalSize { width, height };
                window.set_position(Position::Physical(PhysicalPosition {
                    x: anchor.x + position.x + x,
                    y: anchor.y + position.y + y,
                }))
            },
            WindowEvent::Resize { width, height } => window.set_size(Size::Physical(PhysicalSize { width, height })),
            WindowEvent::Hide => window.hide(),
            WindowEvent::Show => window.show(),
            WindowEvent::Emit { event, payload } => window.emit(event, payload),
        }
        .expect("Failed to process window event");
    }
}
