use anyhow::{Context, Result};
use fig_auth::{get_email, get_token};
use serde::{Deserialize, Serialize};
use std::ops::ControlFlow;
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum FigWebsocketMessageType {
    DotfilesUpdated,
    SettingsUpdated,
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

    info!("Websocket connected");

    Ok(websocket_stream)
}

pub async fn process_websocket(
    websocket_next: &Option<Result<Message, tokio_tungstenite::tungstenite::Error>>,
) -> Result<ControlFlow<()>> {
    match websocket_next {
        Some(next) => match next {
            Ok(websocket_message) => match websocket_message {
                Message::Text(text) => {
                    debug!("message: {:?}", text);

                    let websocket_message_result =
                        serde_json::from_str::<FigWebsocketMessage>(text);

                    match websocket_message_result {
                        Ok(websocket_message) => match websocket_message.websocket_message_type {
                            FigWebsocketMessageType::DotfilesUpdated => {
                                crate::cli::sync::sync_based_on_settings().await?;
                            }
                            FigWebsocketMessageType::SettingsUpdated => {
                                // crate::util::sync::sync(crate::util::sync::Settings {}).await?;
                                info!("Settings updated");
                                warn!("Settings syncing is currently disabled");
                            }
                        },
                        Err(e) => {
                            error!("Could not parse json message: {:?}", e);
                        }
                    }
                    Ok(ControlFlow::Continue(()))
                }
                Message::Close(close_frame) => {
                    match close_frame {
                        Some(close_frame) => {
                            info!("Websocket closed: {:?}", close_frame);
                        }
                        None => info!("Websocket closed"),
                    }

                    Ok(ControlFlow::Break(()))
                }
                Message::Ping(_) => {
                    debug!("Websocket ping");
                    Ok(ControlFlow::Continue(()))
                }
                Message::Pong(_) => {
                    debug!("Websocket pong");
                    Ok(ControlFlow::Continue(()))
                }
                unknown_message => {
                    debug!("Unknown message: {:?}", unknown_message);
                    Ok(ControlFlow::Continue(()))
                }
            },
            Err(err) => {
                error!("Websock next error: {}", err);
                Ok(ControlFlow::Break(()))
            }
        },
        None => {
            info!("Websocket closed");
            Ok(ControlFlow::Break(()))
        }
    }
}
