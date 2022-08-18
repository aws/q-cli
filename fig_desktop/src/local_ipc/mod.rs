mod commands;
mod hooks;

use std::sync::Arc;

use anyhow::{
    anyhow,
    Result,
};
use fig_proto::local::command_response::Response as CommandResponseTypes;
use fig_proto::local::local_message::Type as LocalMessageType;
use fig_proto::local::{
    CommandResponse,
    ErrorResponse,
    LocalMessage,
    SuccessResponse,
};
use fig_util::directories;
use tokio::io::{
    AsyncRead,
    AsyncWrite,
};
use tokio::net::UnixListener;
use tracing::{
    debug,
    error,
    trace,
    warn,
};

use crate::figterm::FigtermState;
use crate::notification::NotificationsState;
use crate::EventLoopProxy;

pub enum LocalResponse {
    Error { code: Option<i32>, message: Option<String> },
    Success(Option<String>),
    Message(Box<CommandResponseTypes>),
}

pub type LocalResult = Result<LocalResponse, LocalResponse>;

pub async fn start_local_ipc(
    figterm_state: Arc<FigtermState>,
    notifications_state: Arc<NotificationsState>,
    proxy: EventLoopProxy,
) -> Result<()> {
    let socket_path = directories::fig_socket_path()?;
    if let Some(parent) = socket_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).expect("Failed creating socket path");
        }
    }

    tokio::fs::remove_file(&socket_path).await.ok();

    let listener = UnixListener::bind(&socket_path)?;

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_local_ipc(
            stream,
            figterm_state.clone(),
            notifications_state.clone(),
            proxy.clone(),
        ));
    }

    Ok(())
}

async fn handle_local_ipc<S: AsyncRead + AsyncWrite + Unpin>(
    mut stream: S,
    _figterm_state: Arc<FigtermState>,
    _notifications_state: Arc<NotificationsState>,
    proxy: EventLoopProxy,
) {
    while let Some(message) = fig_ipc::recv_message::<LocalMessage, _>(&mut stream)
        .await
        .unwrap_or_else(|err| {
            if !err.is_disconnect() {
                error!("Failed receiving local message: {err}");
            }
            None
        })
    {
        trace!("Received local message: {message:?}");
        match message.r#type {
            Some(LocalMessageType::Command(command)) => {
                let response = match command.command {
                    None => LocalResponse::Error {
                        code: None,
                        message: Some("Local ipc command was None".into()),
                    },
                    Some(command) => {
                        use fig_proto::local::command::Command::*;

                        match command {
                            DebugMode(command) => commands::debug(command).await,
                            OpenUiElement(command) => commands::open_ui_element(command, &proxy).await,
                            Quit(command) => commands::quit(command, &proxy).await,
                            Diagnostics(command) => commands::diagnostic(command).await,
                            Logout(_)
                            | TerminalIntegration(_)
                            | ListTerminalIntegrations(_)
                            | Restart(_)
                            | Update(_)
                            | ReportWindow(_)
                            | RestartSettingsListener(_)
                            | RunInstallScript(_)
                            | Build(_)
                            | ResetCache(_)
                            | PromptAccessibility(_)
                            | InputMethod(_)
                            | Uninstall(_) => {
                                debug!("Unhandled command: {command:?}");
                                Err(LocalResponse::Error {
                                    code: None,
                                    message: Some("Unknown command".to_owned()),
                                })
                            },
                        }
                        .unwrap_or_else(|r| r)
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
                    error!("Failed sending local response: {err}");
                    break;
                }
            },
            Some(LocalMessageType::Hook(hook)) => {
                use fig_proto::local::hook::Hook::*;

                macro_rules! legacy_hook {
                    ($name:expr) => {{
                        warn!(
                            "received legacy figterm hook `{}`, please update your figterm version!",
                            $name
                        );
                        Ok(())
                    }};
                }

                if let Err(err) = match hook.hook {
                    Some(EditBuffer(_)) => legacy_hook!("EditBuffer"),
                    Some(CursorPosition(request)) => hooks::caret_position(request, &proxy).await,
                    Some(Prompt(_)) => legacy_hook!("Prompt"),
                    Some(FocusChange(request)) => hooks::focus_change(request, &proxy).await,
                    Some(PreExec(_)) => legacy_hook!("PreExec"),
                    Some(InterceptedKey(_)) => legacy_hook!("InterceptedKey"),
                    Some(FileChanged(request)) => hooks::file_changed(request).await,
                    Some(FocusedWindowData(request)) => hooks::focused_window_data(request, &proxy).await,
                    err => {
                        match &err {
                            Some(unknown) => error!("Unknown hook: {unknown:?}"),
                            None => error!("Hook was none"),
                        }

                        Err(anyhow!("Failed to process hook {err:?}"))
                    },
                } {
                    error!("Error processing hook: {err:?}");
                }
            },
            None => warn!("Received empty local message"),
        }
    }
}
