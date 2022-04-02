#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod api;
mod os;
mod prelude;
mod state;

use crate::state::AppState;
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
    tauri::Builder::default()
        .plugin(constants_plugin())
        .manage(Arc::new(Mutex::new(AppState::default())))
        .setup(|app| {
            //let state = app.state::<AppState>();

            // spawn(handle_ipc(app.handle(), state.inner().clone()));
            // spawn(handle_window(app.handle(), state.inner().clone()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![api::handle_api_request,])
        .run(tauri::generate_context!())
        .expect("error while running tauri application")
}
