use anyhow::{Context, Result};
use fig_auth::{get_email, get_token};
use serde::{Deserialize, Serialize};
use std::{io::Write, ops::ControlFlow};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
enum FigWebsocketMessage {
    DotfilesUpdated,
    #[serde(rename_all = "camelCase")]
    SettingsUpdated {
        settings: serde_json::Value,
        #[serde(with = "time::serde::rfc3339")]
        updated_at: time::OffsetDateTime,
    },
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
                        Ok(websocket_message) => match websocket_message {
                            FigWebsocketMessage::DotfilesUpdated => {
                                crate::cli::sync::sync_based_on_settings().await?;
                            }
                            FigWebsocketMessage::SettingsUpdated {
                                settings,
                                updated_at,
                            } => {
                                // Write settings to disk
                                let path = fig_settings::settings::settings_path()?;

                                info!("Settings updated: Writing settings to disk at {:?}", path);

                                let mut settings_file = std::fs::File::create(&path)?;
                                let settings_json = serde_json::to_string_pretty(&settings)?;
                                settings_file.write_all(settings_json.as_bytes())?;

                                // Write updated_at to disk
                                let path = path.with_extension("updated_at");

                                info!("Settings updated: Writing updated_at to disk at {:?}", path);

                                let mut updated_at_file = std::fs::File::create(path)?;
                                let updated_at_json = serde_json::to_string_pretty(
                                    &updated_at.unix_timestamp_nanos(),
                                )?;
                                updated_at_file.write_all(updated_at_json.as_bytes())?;
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
