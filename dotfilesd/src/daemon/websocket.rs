use std::ops::ControlFlow;

use anyhow::{Result, Context};

use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::{
    auth::{self, Credentials},
    cli::sync,
    daemon::WebsocketAwsToken,
};

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
    let aws_client = auth::get_client("")?;

    let mut creds = Credentials::load_credentials()?;
    creds
        .refresh_credentials(&aws_client, "hkinciohdp1i7h0imdk63a4bv")
        .await?;
    creds.save_credentials()?;

    let websocket_aws_token = match (creds.access_token, creds.id_token) {
        (Some(access_token), Some(id_token)) => WebsocketAwsToken {
            access_token,
            id_token,
        },
        _ => {
            return Err(anyhow::anyhow!("Could not get AWS credentials"));
        }
    };

    let base64_token = base64::encode(&serde_json::to_string(&websocket_aws_token)?);

    let response = reqwest_client
        .get("https://api.fig.io/authenticate/ticket")
        .bearer_auth(&base64_token)
        .send()
        .await?
        .text()
        .await?;

    let mut device_id = crate::util::get_machine_id().context("Cound not get machine_id")?;
    if let Some(email) = creds.email {
        device_id.push(':');
        device_id.push_str(&email);
    }

    let url = url::Url::parse_with_params(
        "wss://api.fig.io/",
        &[("deviceId", &device_id), ("ticket", &response)],
    )?;

    let (websocket_stream, _) = tokio_tungstenite::connect_async(url).await?;

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
                                match sync::sync_all_files().await {
                                    Ok(()) => {
                                        println!("Dotfiles updated");
                                    }
                                    Err(err) => {
                                        println!("Could not update dotfiles: {:?}", err);
                                    }
                                }
                            }
                        },
                        Err(e) => {
                            println!("Could not parse json message: {:?}", e);
                        }
                    }
                }
                Message::Close(close_frame) => {
                    match close_frame {
                        Some(close_frame) => {
                            println!("Websocket closed: {:?}", close_frame);
                        }
                        None => println!("Websocket closed"),
                    }

                    return Ok(ControlFlow::Break(()));
                }
                _ => {}
            },
            Err(err) => {
                println!("{:?}", err);
                return Ok(ControlFlow::Break(()));
            }
        },
        None => {
            println!("Websocket closed");
            return Ok(ControlFlow::Break(()));
        }
    }

    Ok(ControlFlow::Continue(()))
}
