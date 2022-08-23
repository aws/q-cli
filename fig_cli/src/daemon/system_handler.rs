use std::sync::Arc;

use eyre::{
    Result,
    WrapErr,
};
use fig_install::dotfiles::download_and_notify;
use fig_install::plugins::fetch_installed_plugins;
use fig_ipc::{
    recv_message,
    send_message,
};
use fig_proto::daemon::daemon_message::Command;
use fig_proto::daemon::diagnostic_command::DiagnosticPart;
use fig_proto::daemon::diagnostic_response::{
    settings_watcher_status,
    system_socket_status,
    websocket_status,
    SettingsWatcherStatus,
    SystemSocketStatus,
    WebsocketStatus,
};
use fig_proto::daemon::sync_command::SyncType;
use fig_proto::daemon::{
    DaemonMessage,
    DaemonResponse,
};
use fig_telemetry::TrackEvent;
use fig_util::directories;
use parking_lot::RwLock;
use tokio::net::{
    UnixListener,
    UnixStream,
};
use tokio::task::JoinHandle;
use tracing::{
    error,
    info,
    trace,
};
use yaque::Sender;

use super::DaemonStatus;
use crate::util::{
    launch_fig,
    LaunchOptions,
};

async fn spawn_system_handler(mut stream: UnixStream, daemon_status: Arc<RwLock<DaemonStatus>>) -> Result<()> {
    tokio::spawn(async move {
        loop {
            match recv_message::<DaemonMessage, _>(&mut stream).await {
                Ok(Some(message)) => {
                    trace!("Received message: {message:?}");

                    if let Some(command) = &message.command {
                        let response = match command {
                            Command::Diagnostic(diagnostic_command) => {
                                let parts: Vec<_> = diagnostic_command.parts().collect();

                                let daemon_status = daemon_status.read();

                                let time_started_epoch = (parts.is_empty()
                                    || parts.contains(&DiagnosticPart::TimeStartedEpoch))
                                .then_some(daemon_status.time_started);

                                let settings_watcher_status = (parts.is_empty()
                                    || parts.contains(&DiagnosticPart::SettingsWatcherStatus))
                                .then(|| match &daemon_status.settings_watcher_status {
                                    Ok(_) => SettingsWatcherStatus {
                                        status: settings_watcher_status::Status::Ok.into(),
                                        error: None,
                                    },
                                    Err(err) => SettingsWatcherStatus {
                                        status: settings_watcher_status::Status::Error.into(),
                                        error: Some(err.to_string()),
                                    },
                                });

                                let websocket_status = (parts.is_empty()
                                    || parts.contains(&DiagnosticPart::WebsocketStatus))
                                .then(|| match &daemon_status.websocket_status {
                                    Ok(_) => WebsocketStatus {
                                        status: websocket_status::Status::Ok.into(),
                                        error: None,
                                    },
                                    Err(err) => WebsocketStatus {
                                        status: websocket_status::Status::Error.into(),
                                        error: Some(err.to_string()),
                                    },
                                });

                                let system_socket_status = (parts.is_empty()
                                    || parts.contains(&DiagnosticPart::SystemSocketStatus))
                                .then(|| match &daemon_status.system_socket_status {
                                    Ok(_) => SystemSocketStatus {
                                        status: system_socket_status::Status::Ok.into(),
                                        error: None,
                                    },
                                    Err(err) => SystemSocketStatus {
                                        status: system_socket_status::Status::Error.into(),
                                        error: Some(err.to_string()),
                                    },
                                });

                                fig_proto::daemon::new_diagnostic_response(
                                    time_started_epoch,
                                    settings_watcher_status,
                                    websocket_status,
                                    system_socket_status,
                                )
                            },
                            Command::SelfUpdate(_) => {
                                let success = match fig_ipc::command::update_command(true).await {
                                    Ok(()) => {
                                        tokio::task::spawn(async {
                                            tokio::time::sleep(std::time::Duration::from_secs(5)).await;

                                            tokio::task::block_in_place(|| {
                                                launch_fig(LaunchOptions::new().wait_for_activation()).ok();
                                            });
                                        });
                                        true
                                    },
                                    Err(err) => {
                                        error!("Failed to update: {err}");
                                        false
                                    },
                                };
                                fig_proto::daemon::new_self_update_response(success)
                            },
                            Command::Sync(sync_command) => {
                                let update = match sync_command.r#type() {
                                    SyncType::PluginClone => false,
                                    SyncType::PluginUpdate => true,
                                };

                                match download_and_notify(false).await {
                                    Ok(_) => match fetch_installed_plugins(update).await {
                                        Ok(()) => fig_proto::daemon::new_sync_response(Ok(())),
                                        Err(err) => {
                                            error!("Failed to fetch installed plugins: {err}");

                                            fig_proto::daemon::new_sync_response(Err(err.to_string()))
                                        },
                                    },
                                    Err(err) => {
                                        error!("Failed to fetch installed plugins: {err}");

                                        fig_proto::daemon::new_sync_response(Err(err.to_string()))
                                    },
                                }
                            },
                            Command::TelemetryEmitTrack(command) => {
                                let event: TrackEvent = command.into();
                                if command.enqueue.unwrap_or(false) {
                                    if let Ok(dir) = directories::fig_data_dir() {
                                        if let Ok(mut sender) = Sender::open(dir.join("telemetry-track-event-queue")) {
                                            if let Ok(buf) = serde_json::to_vec(&event) {
                                                if sender.send(buf).await.is_ok() {
                                                    continue;
                                                }
                                            }
                                        }
                                    }
                                }
                                tokio::spawn(async move {
                                    if let Err(err) = fig_telemetry::emit_track(event).await {
                                        error!("Failed to emit track: {err}")
                                    }
                                });
                                continue;
                            },
                        };

                        if !message.no_response() {
                            let response = DaemonResponse {
                                id: message.id,
                                response: Some(response),
                            };

                            if let Err(err) = send_message(&mut stream, response).await {
                                error!("Error sending message: {err}");
                            }
                        }
                    }
                },
                Ok(None) => {
                    info!("Received EOF while reading message");
                    break;
                },
                Err(err) => {
                    error!("Error while receiving message: {err}");
                    break;
                },
            }
        }
    });

    Ok(())
}

pub async fn spawn_incoming_system_handler(daemon_status: Arc<RwLock<DaemonStatus>>) -> Result<JoinHandle<()>> {
    let daemon_socket_path = directories::daemon_socket_path()?;

    // Create the system socket directory if it doesn't exist
    if let Some(daemon_socket_dir) = daemon_socket_path.parent() {
        tokio::fs::create_dir_all(daemon_socket_dir)
            .await
            .context("Could not create daemon socket directory")?;
    }

    // Remove the system socket if it already exists
    tokio::fs::remove_file(&daemon_socket_path).await.ok();

    // Bind the system socket
    let daemon_socket = UnixListener::bind(&daemon_socket_path).context("Could not connect to daemon socket")?;

    Ok(tokio::spawn(async move {
        while let Ok((stream, _)) = daemon_socket.accept().await {
            if let Err(err) = spawn_system_handler(stream, daemon_status.clone()).await {
                error!("Error while spawining daemon socket connection handler: {err}");
            }
        }
    }))
}
