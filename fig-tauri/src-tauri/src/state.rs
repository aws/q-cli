#[derive(Default)]
pub struct AppState {
    edit_buffer: EditBuffer,
    cursor_position: Rect,
    window_position: Rect,
    should_intercept: bool,
    os_state: crate::os::native::State,
}

#[derive(Clone, Default)]
pub struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

#[derive(Clone, Default)]
pub struct EditBuffer {
    text: String,
    idx: i64,
}
