#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod api;
mod local;
mod os;
mod state;
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
        .plugin(constants_plugin())
        .setup(|app| {
            tauri::async_runtime::spawn(local::start_local_ipc());
            tauri::async_runtime::spawn(local::figterm::clean_figterm_cache());
            *(STATE.window.lock()) = Some(app.windows().get("autocomplete").unwrap().clone());
            native::init();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![api::handle_api_request])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
