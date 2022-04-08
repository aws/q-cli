use fig_proto::fig::NotificationType;
use hashbrown::HashMap;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::sync::Arc;

use crate::local::figterm::FigTermSession;

pub type AppStateType = Arc<Mutex<AppState>>;

pub static STATE: Lazy<AppStateType> = Lazy::new(<AppStateType>::default);

#[derive(Default, Debug)]
pub struct AppState {
    pub edit_buffer: EditBuffer,
    pub cursor_position: Rect,
    pub _window_position: Rect,
    pub _should_intercept: bool,
    pub subscriptions: HashMap<NotificationType, i64>,
    pub figterm_sessions: HashMap<String, FigTermSession>,
    pub _os_state: crate::os::native::State,
}

#[derive(Clone, Default, Debug)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Default, Debug)]
pub struct EditBuffer {
    pub text: String,
    pub cursor: i64,
}
