#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod api;
mod os;
mod prelude;
mod state;

use std::sync::{Arc, Mutex};

use crate::state::AppState;

fn main() {
    tauri::Builder::default()
        .manage(Arc::new(Mutex::new(AppState::default())))
        // .setup(|app| {
        //     use tauri::Manager;
        //     let state = app.state::<AppState>();
        //     tauri::async_runtime::spawn(handle_ipc(app.handle(), state.inner().clone()));
        //     tauri::async_runtime::spawn(handle_window(app.handle(), state.inner().clone()));
        //     Ok(())
        // })
        .invoke_handler(tauri::generate_handler![api::handle_api_request])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
