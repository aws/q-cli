mod commands;
mod hooks;

use std::sync::Arc;

use anyhow::anyhow;
use fig_proto::local::command_response::Response as CommandResponseTypes;
use fig_proto::local::local_message::Type as LocalMessageType;
use fig_proto::local::{
    CommandResponse,
    ErrorResponse,
    LocalMessage,
    SuccessResponse,
};
use tokio::io::{
    AsyncRead,
    AsyncWrite,
};
use tracing::{
    error,
    trace,
    warn,
};

use crate::figterm::FigtermState;
use crate::window::WindowState;
use crate::{
    native,
    NotificationsState,
};

pub enum LocalResponse {
    Error { code: Option<i32>, message: Option<String> },
    Success(Option<String>),
    Message(Box<CommandResponseTypes>),
}

pub type LocalResult = Result<LocalResponse, LocalResponse>;

pub async fn start_local_ipc(
    figterm_state: Arc<FigtermState>,
    notification_state: Arc<NotificationsState>,
    window_state: Arc<WindowState>,
) {
    let socket_path = fig_ipc::get_fig_socket_path();
    if let Some(parent) = socket_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).expect("Failed creating socket path");
        }
    }

    if socket_path.exists() {
        tokio::fs::remove_file(&socket_path)
            .await
            .expect("Failed clearing socket path");
    }

    let listener = native::Listener::bind(&socket_path);

    while let Ok(stream) = listener.accept().await {
        tokio::spawn(handle_local_ipc(
            stream,
            figterm_state.clone(),
            notification_state.clone(),
            window_state.clone(),
        ));
    }
}

async fn handle_local_ipc<S: AsyncRead + AsyncWrite + Unpin>(
    mut stream: S,
    figterm_state: Arc<FigtermState>,
    notification_state: Arc<NotificationsState>,
    window_state: Arc<WindowState>,
) {
    while let Some(message) = fig_ipc::recv_message::<LocalMessage, _>(&mut stream)
        .await
        .unwrap_or_else(|err| {
            if !err.is_disconnect() {
                error!("Failed receiving local message: {}", err);
            }
            None
        })
    {
        trace!("Received local message: {:?}", message);
        match message.r#type {
            Some(LocalMessageType::Command(command)) => {
                let response = match command.command {
                    None => LocalResponse::Error {
                        code: None,
                        message: Some("Local ipc command was None".to_owned()),
                    },
                    Some(command) => {
                        use fig_proto::local::command::Command::*;

                        match command {
                            DebugMode(command) => commands::debug(command).await.unwrap_or_else(|r| r),
                            _ => LocalResponse::Error {
                                code: None,
                                message: Some("Unknown command".to_owned()),
                            },
                        }
                    },
                };

                let message = {
                    CommandResponse {
                        id: command.id,
                        response: Some(match response {
                            LocalResponse::Error {
                                code: exit_code,
                                message,
                            } => CommandResponseTypes::Error(ErrorResponse { exit_code, message }),
                            LocalResponse::Success(message) => {
                                CommandResponseTypes::Success(SuccessResponse { message })
                            },
                            LocalResponse::Message(m) => *m,
                        }),
                    }
                };

                // TODO: implement AsyncWrite trait for Windows sockets
                if let Err(err) = fig_ipc::send_message(&mut stream, message).await {
                    error!("Failed sending local response: {}", err);
                    break;
                }
            },
            Some(LocalMessageType::Hook(hook)) => {
                use fig_proto::local::hook::Hook;

                match hook.hook {
                    Some(Hook::EditBuffer(request)) => {
                        hooks::edit_buffer(request, figterm_state.clone(), &notification_state, &window_state).await
                    },
                    Some(Hook::CursorPosition(request)) => hooks::caret_position(request, &window_state).await,
                    Some(Hook::Prompt(request)) => hooks::prompt(request).await,
                    Some(Hook::FocusChange(request)) => hooks::focus_change(request).await,
                    Some(Hook::PreExec(request)) => hooks::pre_exec(request).await,
                    Some(Hook::InterceptedKey(request)) => {
                        hooks::intercepted_key(request, &notification_state, &window_state).await
                    },
                    Some(Hook::FileChanged(request)) => hooks::file_changed(request).await,
                    err => {
                        match &err {
                            Some(unknown) => error!("Unknown hook: {:?}", unknown),
                            None => error!("Hook was none"),
                        }

                        Err(anyhow!("Failed to process hook {err:?}"))
                    },
                }
                .unwrap();
            },
            None => warn!("Received empty local message"),
        }
    }
}
