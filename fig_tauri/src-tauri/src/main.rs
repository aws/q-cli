#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod api;
mod figterm;
mod icons;
mod local_ipc;
mod native;
mod tray;
mod utils;
mod window;

use std::sync::Arc;

use dashmap::DashMap;
use fig_proto::fig::NotificationType;
use figterm::FigtermState;
use native::NativeState;
use parking_lot::RwLock;
use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};
use tokio::sync::mpsc;
use window::WindowState;

const FIG_PROTO_MESSAGE_RECIEVED: &str = "FigProtoMessageRecieved";
// TODO: Add constants
const JAVASCRIPT_INIT: &str = r#"
console.log("[fig] declaring constants...")

if (!window.fig) {
    window.fig = {}
}

if (!window.fig.constants) {
    window.fig.constants = {}
}
"#;

#[derive(Debug, Default)]
pub struct DebugState {
    pub debug_lines: RwLock<Vec<String>>,
    pub color: RwLock<Option<String>>,
}

#[derive(Debug, Default)]
pub struct InterceptState {
    pub intercept_bound_keystrokes: RwLock<bool>,
    pub intercept_global_keystrokes: RwLock<bool>,
}

#[derive(Debug, Default)]
pub struct NotificationsState {
    subscriptions: DashMap<NotificationType, i64, fnv::FnvBuildHasher>,
}

fn constants_plugin<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("constants")
        .js_init_script(JAVASCRIPT_INIT.to_string())
        .build()
}

fn main() {
    fig_log::init_logger("fig_tauri.log").expect("Failed to initialize logger");

    let (send, recv) = mpsc::unbounded_channel();

    let debug_state = Arc::new(DebugState::default());
    let figterm_state = Arc::new(FigtermState::default());
    let intercept_state = InterceptState::default();
    let native_state = NativeState::new(send.clone());
    let notifications_state = Arc::new(NotificationsState::default());

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![api::handle_api_request])
        .setup({
            let figterm_state = figterm_state.clone();
            let notifications_state = notifications_state.clone();
            |app| {
                let window = app
                    .get_window("autocomplete")
                    .expect("Failed to acquire autocomplete window");
                let window_state = Arc::new(WindowState::new(&window, send));
                app.manage(window_state.clone());

                tauri::async_runtime::spawn(figterm::clean_figterm_cache(figterm_state.clone()));

                tauri::async_runtime::spawn(local_ipc::start_local_ipc(
                    figterm_state,
                    notifications_state,
                    window_state.clone(),
                ));

                tauri::async_runtime::spawn(window::handle_window(window, recv, window_state));

                Ok(())
            }
        })
        .plugin(constants_plugin())
        .system_tray(tray::create_tray())
        .on_system_tray_event({
            let debug_state = debug_state.clone();
            let figterm_state = figterm_state.clone();
            move |app, event| {
                tray::handle_tray_event(app, event, debug_state.clone(), figterm_state.clone())
            }
        })
        .register_uri_scheme_protocol("fig", icons::handle)
        .manage(debug_state)
        .manage(figterm_state)
        .manage(intercept_state)
        .manage(native_state)
        .manage(notifications_state)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
