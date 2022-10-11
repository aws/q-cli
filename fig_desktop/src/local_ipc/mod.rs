pub(crate) mod commands;
mod hooks;

use std::sync::Arc;

use anyhow::{
    anyhow,
    Result,
};
use fig_ipc::{
    BufferedUnixStream,
    RecvMessage,
    SendMessage,
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
use tokio::net::UnixListener;
use tracing::{
    debug,
    error,
    trace,
    warn,
};

use crate::platform::PlatformState;
use crate::EventLoopProxy;

pub enum LocalResponse {
    Error { code: Option<i32>, message: Option<String> },
    Success(Option<String>),
    Message(Box<CommandResponseTypes>),
}

pub type LocalResult = Result<LocalResponse, LocalResponse>;

pub async fn start_local_ipc(platform_state: Arc<PlatformState>, proxy: EventLoopProxy) -> Result<()> {
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
            BufferedUnixStream::new(stream),
            platform_state.clone(),
            proxy.clone(),
        ));
    }

    Ok(())
}

async fn handle_local_ipc(mut stream: BufferedUnixStream, platform_state: Arc<PlatformState>, proxy: EventLoopProxy) {
    while let Some(message) = stream.recv_message::<LocalMessage>().await.unwrap_or_else(|err| {
        if !err.is_disconnect() {
            error!("Failed receiving local message: {err}");
        }
        None
    }) {
        trace!("Received local message: {message:?}");
        match message.r#type {
            Some(LocalMessageType::Command(command)) => {
                warn!("COMMAND {command:?}");
                let response = match command.command {
                    None => LocalResponse::Error {
                        code: None,
                        message: Some("Local ipc command was None".into()),
                    },
                    Some(command) => {
                        use fig_proto::local::command::Command::*;

                        match command {
                            DebugMode(command) => commands::debug(command, &proxy).await,
                            OpenUiElement(command) => commands::open_ui_element(command, &proxy).await,
                            Quit(command) => commands::quit(command, &proxy).await,
                            Diagnostics(command) => commands::diagnostic(command).await,
                            OpenBrowser(command) => commands::open_browser(command).await,
                            PromptAccessibility(_) => commands::prompt_for_accessibility_permission().await,
                            Logout(_)
                            | TerminalIntegration(_)
                            | ListTerminalIntegrations(_)
                            | Restart(_)
                            | ReportWindow(_)
                            | RestartSettingsListener(_)
                            | RunInstallScript(_)
                            | Update(_)
                            | Build(_)
                            | ResetCache(_)
                            | InputMethod(_) => {
                                debug!(?command, "Unhandled command");
                                Err(LocalResponse::Error {
                                    code: None,
                                    message: Some("Unknown command".to_owned()),
                                })
                            },
                        }
                        .unwrap_or_else(|r| r)
                    },
                };

                match command.no_response {
                    Some(true) => {},
                    _ => {
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
                        if let Err(err) = stream.send_message(message).await {
                            error!(%err, "Failed sending local response");
                            break;
                        }
                    },
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
                    Some(FocusedWindowData(request)) => {
                        hooks::focused_window_data(request, &platform_state, &proxy).await
                    },
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
