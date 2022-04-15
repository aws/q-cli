use anyhow::anyhow;
use fig_proto::local::{
    command_response::Response as CommandResponseTypes, local_message::Type as LocalMessageType,
    CommandResponse, ErrorResponse, LocalMessage, SuccessResponse,
};
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::{error, trace, warn};

use crate::os::native;

mod commands;
pub mod figterm;
mod hooks;

#[allow(unused)]
pub enum ResponseKind {
    Error((Option<i32>, Option<String>)),
    Success(Option<String>),
    Message(Box<CommandResponseTypes>),
}

pub type ResponseResult = Result<ResponseKind, ResponseKind>;

impl ResponseKind {
    #[allow(unused)]
    pub fn error_exit_code(exit_code: i32) -> Self {
        Self::Error((Some(exit_code), None))
    }

    pub fn error_message<S: ToString>(message: S) -> Self {
        Self::Error((None, Some(message.to_string())))
    }

    #[allow(unused)]
    pub fn success() -> Self {
        Self::Success(None)
    }

    #[allow(unused)]
    pub fn success_message<S: ToString>(message: S) -> Self {
        Self::Success(Some(message.to_string()))
    }
}

pub async fn start_local_ipc() {
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
        tokio::spawn(handle_local_ipc(stream))
            .await
            .expect("Failed to spawn ipc handler");
    }
}

async fn handle_local_ipc<S: AsyncRead + AsyncWrite + Unpin>(mut stream: S) {
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
                macro_rules! route {
                    ($($struct: ident => $func: path)*) => {
                        match command.command {
                            $(
                                Some(fig_proto::local::command::Command::$struct(request)) => $func(request).await,
                            )*
                            _ => Err(ResponseKind::error_message("Unknown command"))
                        }
                    }
                }

                let response = route! {
                    DebugMode => commands::manage::debug
                }
                .unwrap_or_else(|s| s);

                let message = {
                    CommandResponse {
                        id: command.id,
                        response: Some(match response {
                            ResponseKind::Error((exit_code, message)) => {
                                CommandResponseTypes::Error(ErrorResponse { exit_code, message })
                            }
                            ResponseKind::Success(message) => {
                                CommandResponseTypes::Success(SuccessResponse { message })
                            }
                            ResponseKind::Message(m) => *m,
                        }),
                    }
                };

                // TODO: implement AsyncWrite trait for Windows sockets
                if let Err(err) = fig_ipc::send_message(&mut stream, message).await {
                    error!("Failed sending local response: {}", err);
                    break;
                }
            }
            Some(LocalMessageType::Hook(hook)) => {
                macro_rules! route {
                    ($($struct: ident => $func: path)*) => {
                        match hook.hook {
                            $(
                                Some(fig_proto::local::hook::Hook::$struct(request)) => $func(request).await,
                            )*
                            s => Err(anyhow!("Unknown hook {:?}", s))
                        }
                    }
                }

                if let Err(err) = route! {
                    EditBuffer => hooks::state::edit_buffer
                    CursorPosition => hooks::state::cursor_position
                    Prompt => hooks::state::prompt
                    FocusChange => hooks::state::focus_change
                } {
                    error!("Failed processing hook: {}", err);
                }
            }
            None => warn!("Received empty local message"),
        }
    }
}
