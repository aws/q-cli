use anyhow::{anyhow, Context, Result};
use fig_auth::{get_email, get_token, refresh_credentals};
use fig_settings::{api_host, ws_host};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::Write;
use time::format_description::well_known::Rfc3339;
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info};

use crate::daemon::scheduler::{Scheduler, SyncDotfiles};

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
    info!("Connecting to websocket");

    let reqwest_client = reqwest::Client::new();

    let token = get_token().await?;

    let api_host = api_host();

    let url = Url::parse(&format!("{api_host}/authenticate/ticket"))?;

    let response = reqwest_client
        .get(url.clone())
        .bearer_auth(&token)
        .send()
        .await?
        .error_for_status();

    let text = match response {
        Ok(response) => response.text().await?,
        Err(_) => {
            // Retry after manually refreshing the credentals
            refresh_credentals().await?;
            let token = get_token().await?;

            reqwest_client
                .get(url.clone())
                .bearer_auth(&token)
                .send()
                .await?
                .error_for_status()?
                .text()
                .await?
        }
    };

    let mut device_id = crate::util::get_machine_id().context("Cound not get machine_id")?;
    if let Some(email) = get_email() {
        device_id.push(':');
        device_id.push_str(&email);
    }

    let url = Url::parse_with_params(&ws_host(), &[("deviceId", &device_id), ("ticket", &text)])?;

    let (websocket_stream, _) = tokio_tungstenite::connect_async(url).await?;

    info!("Websocket connected");

    Ok(websocket_stream)
}

pub async fn process_websocket(
    websocket_next: &Option<Result<Message, tokio_tungstenite::tungstenite::Error>>,
    scheduler: &mut Scheduler,
) -> Result<()> {
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
                                scheduler.schedule_now(SyncDotfiles);
                            }
                            FigWebsocketMessage::SettingsUpdated {
                                settings,
                                updated_at,
                            } => {
                                // Write settings to disk
                                let path = fig_settings::settings::settings_path()
                                    .context("Could not get settings path")?;

                                info!("Settings updated: Writing settings to disk at {:?}", path);

                                let mut settings_file = std::fs::File::create(&path)?;
                                let settings_json = serde_json::to_string_pretty(&settings)?;
                                settings_file.write_all(settings_json.as_bytes())?;

                                if let Ok(updated_at) = updated_at.format(&Rfc3339) {
                                    fig_settings::state::set_value(
                                        "settings.updatedAt",
                                        json!(updated_at),
                                    )
                                    .ok();
                                }
                            }
                        },
                        Err(e) => {
                            error!("Could not parse json message: {:?}", e);
                        }
                    }
                    Ok(())
                }
                Message::Close(close_frame) => match close_frame {
                    Some(close_frame) => {
                        info!("Websocket close frame: {:?}", close_frame);
                        Err(anyhow!("Websocket close frame: {:?}", close_frame))
                    }
                    None => {
                        info!("Websocket close frame");
                        Err(anyhow!("Websocket close frame"))
                    }
                },
                Message::Ping(_) => {
                    debug!("Websocket ping");
                    Ok(())
                }
                Message::Pong(_) => {
                    debug!("Websocket pong");
                    Ok(())
                }
                unknown_message => {
                    debug!("Unknown message: {:?}", unknown_message);
                    Ok(())
                }
            },
            Err(err) => {
                error!("Websock next error: {:?}", err);
                Err(anyhow!("Websock next error: {:?}", err))
            }
        },
        None => {
            info!("Websocket closed");
            Err(anyhow!("Websocket closed"))
        }
    }
}
