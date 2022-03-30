#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

pub mod os;

use serde::Serialize;
use std::fs::remove_file;
use std::sync::Arc;
use tauri::async_runtime::{spawn, Mutex};
use tauri::{AppHandle, Manager, Runtime};
use tokio::join;
use tokio::net::UnixListener;

#[derive(Clone, Serialize, Default)]
struct EditBuffer {
    text: String,
    idx: i64,
}

#[derive(Clone, Serialize, Default)]
struct CursorPosition {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

#[derive(Default)]
struct AppState {
    edit_buffer: EditBuffer,
    cursor_position: CursorPosition,
}

#[tauri::command]
fn get_edit_buffer() -> EditBuffer {
    EditBuffer {
        text: "Foo".to_string(),
        idx: 0,
    }
}

#[tauri::command]
fn get_cursor_position() -> CursorPosition {
    CursorPosition {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    }
}

async fn handle_ipc<R: tauri::Runtime>(tauri_app: AppHandle<R>, state: Arc<Mutex<AppState>>) {
    let app_socket_path = fig_ipc::get_fig_socket_path();
    let linux_socket_path = fig_ipc::get_fig_linux_socket_path();

    if app_socket_path.exists() {
        remove_file(&app_socket_path).expect("Failed deleting existing Fig socket");
    }

    if linux_socket_path.exists() {
        remove_file(&linux_socket_path).expect("Failed deleting existing Fig Linux socket");
    }

    let app_state = state.clone();
    let tauri_app_clone = tauri_app.clone();
    let app_task = spawn(async move {
        let app_listener = UnixListener::bind(&app_socket_path).expect("Failed binding Fig socket");
        tokio::spawn(async move {
            loop {
                if let Ok((mut stream, _)) = app_listener.accept().await {
                    let app_state = app_state.clone();
                    let tauri_app = tauri_app_clone.clone();
                    tokio::spawn(async move {
                        loop {
                            use fig_proto::local::*;
                            match fig_ipc::recv_message::<fig_proto::local::LocalMessage, _>(
                                &mut stream,
                            )
                            .await
                            {
                                Ok(Some(LocalMessage {
                                    r#type:
                                        Some(local_message::Type::Hook(Hook {
                                            hook:
                                                Some(hook::Hook::EditBuffer(EditBufferHook {
                                                    text,
                                                    cursor,
                                                    ..
                                                })),
                                        })),
                                })) => {
                                    let mut handle = app_state.lock().await;
                                    handle.edit_buffer = EditBuffer {
                                        text: text.clone(),
                                        idx: cursor,
                                    };
                                    tauri_app.emit_all(
                                        "update-edit-buffer",
                                        EditBuffer { text, idx: cursor },
                                    );
                                }
                                Ok(None) => break,
                                Err(err) => {
                                    println!("error receiving message: {:?}", err);
                                    break;
                                }
                                _ => {}
                            }
                        }
                    });
                }
            }
        })
    });

    let linux_state = state;
    let linux_task = spawn(async move {
        let linux_listener =
            UnixListener::bind(&linux_socket_path).expect("Failed binding Fig socket");
        tokio::spawn(async move {
            loop {
                if let Ok((mut stream, _)) = linux_listener.accept().await {
                    let linux_state = linux_state.clone();
                    let tauri_app = tauri_app.clone();
                    tokio::spawn(async move {
                        loop {
                            use fig_proto::linux::*;
                            match fig_ipc::recv_message::<fig_proto::linux::AppCommand, _>(
                                &mut stream,
                            )
                            .await
                            {
                                Ok(Some(AppCommand {
                                    command:
                                        Some(app_command::Command::SetCursorPosition(
                                            SetCursorPositionCommand {
                                                x,
                                                y,
                                                width,
                                                height,
                                            },
                                        )),
                                })) => {
                                    let mut handle = linux_state.lock().await;
                                    handle.cursor_position = CursorPosition {
                                        x,
                                        y,
                                        width,
                                        height,
                                    };
                                    tauri_app.emit_all(
                                        "update-cursor-position",
                                        CursorPosition {
                                            x,
                                            y,
                                            width,
                                            height,
                                        },
                                    );
                                }
                                Ok(None) => break,
                                Err(err) => {
                                    println!("error receiving message: {:?}", err);
                                    break;
                                }
                                _ => {}
                            }
                        }
                    });
                };
            }
        })
    });

    let (app_join, linux_join) = join![app_task, linux_task];

    app_join.expect("Error while running app socket task");
    linux_join.expect("Error while running linux socket task");
}

fn main() {
    tauri::Builder::default()
        .manage(Arc::new(Mutex::new(AppState::default())))
        .invoke_handler(tauri::generate_handler![
            get_edit_buffer,
            get_cursor_position
        ])
        .setup(|app| {
            let state = app.state::<Arc<Mutex<AppState>>>();
            spawn(handle_ipc(app.handle(), state.inner().clone()));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
