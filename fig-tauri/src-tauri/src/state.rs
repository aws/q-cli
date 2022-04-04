use std::sync::Arc;

use tauri::async_runtime::Mutex;

pub type AppStateType = Arc<Mutex<AppState>>;

#[derive(Default)]
pub struct AppState {
    pub _edit_buffer: EditBuffer,
    pub _cursor_position: Rect,
    pub _window_position: Rect,
    pub _should_intercept: bool,
    pub _os_state: crate::os::native::State,
}

#[derive(Clone, Default)]
pub struct Rect {
    pub _x: i32,
    pub _y: i32,
    pub _width: i32,
    pub _height: i32,
}

#[derive(Clone, Default)]
pub struct EditBuffer {
    pub _text: String,
    pub _idx: i64,
}
