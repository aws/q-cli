use dashmap::DashMap;
use fig_proto::fig::NotificationType;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::Window;

use crate::local::figterm::FigTermSession;

pub type AppStateType = Arc<AppState>;

pub static STATE: Lazy<AppStateType> = Lazy::new(<AppStateType>::default);

#[derive(Default, Debug)]
pub struct AppState {
    pub cursor_position: Mutex<Rect>,
    pub _window_position: Rect,
    pub _should_intercept: bool,
    pub subscriptions: DashMap<NotificationType, i64>,
    pub figterm_sessions: DashMap<String, FigTermSession>,
    pub window: Mutex<Option<Window>>,
    pub ui_state: Mutex<UIState>,
    pub anchor: Mutex<Point>,
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

impl Default for UIState {
    fn default() -> Self {
        Self::Unfocused
    }
}
