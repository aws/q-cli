use parking_lot::RwLock;
use tracing::debug;
use wry::application::dpi::{
    PhysicalPosition,
    PhysicalSize,
    Position,
};
use wry::webview::WebView;

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

impl FigWindowEvent {
    pub fn handle(self, window: &WebView, state: &WindowState) {
        match self {
            FigWindowEvent::Reanchor { x, y } => {
                let position = state.position.read();
                let caret_position = state.caret_position.read();
                *state.anchor.write() = PhysicalPosition { x, y };
                window.window().set_outer_position(Position::Physical(PhysicalPosition {
                    x: x + position.x + caret_position.x,
                    y: y + position.y + caret_position.y,
                }))
            },
            FigWindowEvent::Reposition { x, y } => {
                let anchor = state.anchor.read();
                let caret_position = state.caret_position.read();
                *state.position.write() = PhysicalPosition {
                    x: caret_position.x,
                    y: caret_position.y,
                };
                debug!(
                    "x {x} y {y} anchor.x {} anchor.y {} caret_position.x {} caret_position.y {}",
                    anchor.x, anchor.y, caret_position.x, caret_position.y
                );
                window.window().set_outer_position(Position::Physical(PhysicalPosition {
                    x: caret_position.x,
                    y: caret_position.y,
                }))
            },
            FigWindowEvent::UpdateCaret { x, y, width, height } => {
                let anchor = PhysicalPosition { x, y };
                let position = state.position.read();
                *state.caret_position.write() = PhysicalPosition { x, y };
                *state.caret_size.write() = PhysicalSize { width, height };
                window
                    .window()
                    .set_outer_position(Position::Physical(PhysicalPosition { x, y }))
            },
            FigWindowEvent::Resize { width, height } => window.window().set_inner_size(PhysicalSize { width, height }),
            FigWindowEvent::Hide => window.window().set_visible(false),
            FigWindowEvent::Show => {
                window.window().set_visible(true);
                window.window().set_always_on_top(true);
            },
            FigWindowEvent::Emit { event, payload } => {
                window
                    .evaluate_script(&format!(
                        "document.dispatchEvent(new CustomEvent('{event}', {{'detail': `{payload}`}}))"
                    ))
                    .unwrap();
                window
                    .evaluate_script(&format!("console.log('Executing {event}')"))
                    .unwrap();
            },
            FigWindowEvent::Api { payload } => {},
        }
    }
}

#[derive(Debug)]
pub struct WindowState {
    anchor: RwLock<PhysicalPosition<i32>>,
    position: RwLock<PhysicalPosition<i32>>,
    caret_position: RwLock<PhysicalPosition<i32>>,
    caret_size: RwLock<PhysicalSize<i32>>,
}

impl WindowState {
    pub fn new(window: &WebView) -> Self {
        Self {
            anchor: RwLock::new(PhysicalPosition::default()),
            position: RwLock::new(
                window
                    .window()
                    .inner_position()
                    .expect("Failed to acquire window position"),
            ),
            caret_position: RwLock::new(PhysicalPosition::default()),
            caret_size: RwLock::new(PhysicalSize::default()),
        }
    }
}
