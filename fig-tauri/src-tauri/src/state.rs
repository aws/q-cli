use fig_proto::fig::NotificationType;
use hashbrown::HashMap;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::sync::Arc;

pub type AppStateType = Arc<Mutex<AppState>>;

pub static STATE: Lazy<AppStateType> = Lazy::new(|| <AppStateType>::default());

#[derive(Default, Debug)]
pub struct AppState {
    pub _edit_buffer: EditBuffer,
    pub _cursor_position: Rect,
    pub _window_position: Rect,
    pub _should_intercept: bool,
    pub subscriptions: HashMap<NotificationType, i64>,
    pub _os_state: crate::os::native::State,
}

#[derive(Clone, Default, Debug)]
pub struct Rect {
    pub _x: i32,
    pub _y: i32,
    pub _width: i32,
    pub _height: i32,
}

#[derive(Clone, Default, Debug)]
pub struct EditBuffer {
    pub _text: String,
    pub _idx: i64,
}
