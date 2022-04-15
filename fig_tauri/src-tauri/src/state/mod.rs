pub mod figterm;

use dashmap::DashMap;
use fig_proto::fig::NotificationType;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::sync::{Arc, RwLock};
use tauri::Window;

use self::figterm::FigtermState;

pub type AppStateType = Arc<AppState>;

pub static STATE: Lazy<AppStateType> = Lazy::new(<AppStateType>::default);

#[derive(Default, Debug)]
pub struct AppState {
    pub cursor_position: Mutex<Rect>,
    pub _window_position: Rect,
    pub _should_intercept: bool,
    pub subscriptions: DashMap<NotificationType, i64>,
    pub figterm_state: FigtermState,
    pub window: RwLock<Option<Window>>,
    pub anchor: RwLock<Point>,
    pub _os_state: crate::os::native::State,
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UIState {
    Focused {
        caret: Rect,
        window: Rect,
        screen: Rect,
    },
    Unfocused,
}
