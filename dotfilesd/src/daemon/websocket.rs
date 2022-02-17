use crate::{
    cli::sync::{self, notify_terminals, SyncWhen},
    daemon::daemon_log,
    util::settings::Settings,
};

use anyhow::{Context, Result};
use fig_auth::{get_email, get_token};
use serde::{Deserialize, Serialize};
use std::ops::ControlFlow;
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum FigWebsocketMessageType {
    DotfilesUpdated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FigWebsocketMessage {
    #[serde(rename = "type")]
    websocket_message_type: FigWebsocketMessageType,
}

pub async fn connect_to_fig_websocket() -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
    let reqwest_client = reqwest::Client::new();

    let token = get_token().await?;

    let response = reqwest_client
        .get("https://api.fig.io/authenticate/ticket")
        .bearer_auth(&token)
        .send()
        .await?
        .text()
        .await?;

    let mut device_id = crate::util::get_machine_id().context("Cound not get machine_id")?;
    if let Some(email) = get_email() {
        device_id.push(':');
        device_id.push_str(&email);
    }

    let url = url::Url::parse_with_params(
        "wss://api.fig.io/",
        &[("deviceId", &device_id), ("ticket", &response)],
    )?;

    let (websocket_stream, _) = tokio_tungstenite::connect_async(url).await?;

    daemon_log("Websocket connected");

    Ok(websocket_stream)
}

pub async fn process_websocket(
    websocket_next: &Option<Result<Message, tokio_tungstenite::tungstenite::Error>>,
) -> Result<ControlFlow<()>> {
    match websocket_next {
        Some(next) => match next {
            Ok(websocket_message) => match websocket_message {
                Message::Text(text) => {
                    let websocket_message_result =
                        serde_json::from_str::<FigWebsocketMessage>(text);

                    match websocket_message_result {
                        Ok(websocket_message) => match websocket_message.websocket_message_type {
                            FigWebsocketMessageType::DotfilesUpdated => {
                                let sync_when = if let Ok(settings) = Settings::load() {
                                    settings
                                        .get_setting()
                                        .map(|setting| {
                                            if setting
                                                .get("dotfiles.syncImmediately")
                                                .map(|val| val.as_bool())
                                                .flatten()
                                                == Some(true)
                                            {
                                                SyncWhen::Immediately
                                            } else {
                                                SyncWhen::Later
                                            }
                                        })
                                        .unwrap_or(SyncWhen::Later)
                                } else {
                                    SyncWhen::Later
                                };

                                match sync::sync_all_files(sync_when).await {
                                    Ok(()) => match sync_when {
                                        SyncWhen::Immediately => {
                                            notify_terminals()?;
                                            daemon_log("Dotfiles updated");
                                        }
                                        SyncWhen::Later => {
                                            daemon_log("New dotfiles available");
                                        }
                                    },
                                    Err(err) => {
                                        daemon_log(&format!("Could not sync dotfiles: {:?}", err));
                                    }
                                }
                            }
                        },
                        Err(e) => {
                            daemon_log(&format!("Could not parse json message: {:?}", e));
                        }
                    }
                }
                Message::Close(close_frame) => {
                    match close_frame {
                        Some(close_frame) => {
                            daemon_log(&format!("Websocket closed: {:?}", close_frame));
                        }
                        None => daemon_log("Websocket closed"),
                    }

                    return Ok(ControlFlow::Break(()));
                }
                _ => {}
            },
            Err(err) => {
                daemon_log(&format!("{:?}", err));
                return Ok(ControlFlow::Break(()));
            }
        },
        None => {
            daemon_log("Websocket closed");
            return Ok(ControlFlow::Break(()));
        }
    }

    Ok(ControlFlow::Continue(()))
}
