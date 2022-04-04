#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod api;
mod local;
mod os;
mod state;

use crate::state::{AppState, AppStateType};
use std::sync::{Arc, Mutex};
use tauri::{
    plugin::{Builder, TauriPlugin},
    utils::config::WindowUrl,
    window::WindowBuilder,
    Manager, Runtime,
};

fn declare_constants() -> String {
    let mut script: Vec<&str> = Vec::new();

    script.push(
        r#"
    if (!window.fig) {
        window.fig = {}
    }
    
    if (!window.fig.constants) {
        window.fig.constants = {}
    }
    "#,
    );

    script.push(r#"console.log("[fig] declaring constants...")"#);

    script.join("\n")
}

fn constants_plugin<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("constants")
        .js_init_script(declare_constants())
        .build()
}

fn main() {
    fig_log::init_logger("fig-tauri.log").unwrap();

    tauri::Builder::default()
        .plugin(constants_plugin())
        .manage(Arc::new(Mutex::new(AppState::default())))
        .setup(|app| {
            let state = app.state::<AppStateType>();
            tauri::async_runtime::spawn(local::start_local_ipc(state.inner().clone()));
            Ok(())
        })
        .on_window_event(|_| println!("window event"))
        .invoke_handler(tauri::generate_handler![api::handle_api_request])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
