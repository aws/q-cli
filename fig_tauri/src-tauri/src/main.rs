#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod api;
mod icons;
mod local;
mod os;
mod state;
pub mod tray;
mod utils;

use crate::{os::native, state::STATE};
use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

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

fn constants_plugin<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("constants")
        .js_init_script(JAVASCRIPT_INIT.to_string())
        .build()
}

fn main() {
    fig_log::init_logger("fig_tauri.log").unwrap();

    tauri::Builder::default()
        .system_tray(tray::create_tray())
        .on_system_tray_event(tray::handle_tray_event)
        .plugin(constants_plugin())
        .setup(|app| {
            tauri::async_runtime::spawn(local::start_local_ipc());
            tauri::async_runtime::spawn(local::figterm::clean_figterm_cache());
            *(STATE.window.write().unwrap()) =
                Some(app.windows().get("autocomplete").unwrap().clone());
            native::init();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![api::handle_api_request])
        .register_uri_scheme_protocol("fig", icons::handle)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
