#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod api;
mod local;
mod os;
mod state;

use std::sync::{Arc, Mutex};

use tauri::Manager;

use crate::state::{AppState, AppStateType};

fn main() {
    tauri::Builder::default()
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
