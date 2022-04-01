#[derive(Default)]
pub struct AppState {
    _edit_buffer: EditBuffer,
    _cursor_position: Rect,
    _window_position: Rect,
    _should_intercept: bool,
    _os_state: crate::os::native::State,
}

#[derive(Clone, Default)]
pub struct Rect {
    _x: i32,
    _y: i32,
    _width: i32,
    _height: i32,
}

#[derive(Clone, Default)]
pub struct EditBuffer {
    _text: String,
    _idx: i64,
}
