#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod api;
mod os;
mod prelude;
mod state;

use std::sync::{Arc, Mutex};

use tauri::Manager;

use crate::state::AppState;

fn main() {
    fn main() {
        tauri::Builder::default()
            .manage(Arc::new(Mutex::new(AppState::default())))
            .setup(|app| {
                let state = app.state::<AppState>();

                // spawn(handle_ipc(app.handle(), state.inner().clone()));
                // spawn(handle_window(app.handle(), state.inner().clone()));

                Ok(())
            })
            .invoke_handler(tauri::generate_handler![api::handle_api_request])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    }
}
