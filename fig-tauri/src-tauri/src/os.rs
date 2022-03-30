use std::sync::{Arc, Mutex};

use tauri::AppHandle;

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

#[cfg(target_os = "macos")]
pub struct MacOsState {}

#[cfg(target_os = "linux")]
pub struct LinuxState {}

#[cfg(target_os = "windows")]
pub struct WindowsState {
    window_id: u32,
    process_id: u32,
}

#[derive(Default)]
pub struct AppState<S> {
    edit_buffer: EditBuffer,
    cursor_position: Rect,
    window_position: Rect,
    should_intercept: bool,
    os_state: S,
}

/// local.proto figterm.proto
///
/// Figterm, fig cli, daemon, incomming, ibus, etc
///
/// fig.socket

#[cfg(target_os = "linux")]
pub async fn handle_local_ipc<R: tauri::Runtime>(
    tauri_app: AppHandle<R>,
    state: Arc<Mutex<AppState<LinuxState>>>,
) {
}

#[cfg(target_os = "windows")]
pub async fn handle_local_ipc<R: tauri::Runtime>(
    tauri_app: AppHandle<R>,
    state: Arc<Mutex<AppState<WindowsState>>>,
) {
    
}

// fig.proto

pub async fn handle_api_ipc<R: tauri::Runtime>(
    tauri_app: AppHandle<R>,
    state: Arc<Mutex<AppState<()>>>,
) {
    // Emitting subscriptions
}

#[tauri::command]
pub fn execute_api_cmd(event: Vec<u8>) {
    // Deser

    // Suscribe to events
    // Send event from web api
}

// Window manager event loop

#[cfg(target_os = "linux")]
pub async fn handle_window<R: tauri::Runtime>(
    tauri_app: AppHandle<R>,
    state: Arc<Mutex<AppState<LinuxState>>>,
) {
    // X11 or wayland
}

#[cfg(target_os = "windows")]
pub async fn handle_window<R: tauri::Runtime>(
    tauri_app: AppHandle<R>,
    state: Arc<Mutex<AppState<WindowState>>>,
) {
    // Listinng to windows api
}
