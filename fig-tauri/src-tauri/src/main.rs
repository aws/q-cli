#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod os;

use std::sync::{Arc, Mutex};

use os::*;

fn main() {
    fn main() {
        tauri::Builder::default()
            .manage(Arc::new(Mutex::new(AppState::default())))
            .setup(|app| {
                let mut state = None;

                cfg_if::cfg_if! {
                    if #[cfg(target_os = "linux")] {
                        state = Some(app.state::<Arc<Mutex<AppState<LinuxState>>>>());
                    } else if #[cfg(target_os = "windows")] {
                        state = Some(app.state::<Arc<Mutex<AppState<WindowsState>>>>());
                    }
                };

                let state = state.expect("Unsupported platform");

                spawn(handle_ipc(app.handle(), state.inner().clone()));
                spawn(handle_window(app.handle(), state.inner().clone()));

                Ok(())
            })
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    }
}
