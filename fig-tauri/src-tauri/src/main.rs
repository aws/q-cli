#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod api;
mod local;
mod os;
mod state;

use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
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
        .setup(|_| {
            tauri::async_runtime::spawn(local::start_local_ipc());
            tauri::async_runtime::spawn(local::figterm::clean_figterm_cache());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![api::handle_api_request])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
