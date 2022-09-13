use std::io::Write;
use std::time::Duration;

use eyre::{
    bail,
    eyre,
    Result,
    WrapErr,
};
use fig_auth::get_email;
use fig_ipc::local::{
    send_hook_to_socket,
    update_command,
};
use fig_proto::hooks::new_event_hook;
use fig_request::reqwest::StatusCode;
use fig_request::Request;
use fig_settings::{
    settings,
    ws_host,
};
use fig_util::system_info::get_system_id;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use time::format_description::well_known::Rfc3339;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{
    MaybeTlsStream,
    WebSocketStream,
};
use tracing::{
    debug,
    error,
    info,
    warn,
};
use url::Url;

use crate::daemon::scheduler::{
    Scheduler,
    SyncDotfiles,
};
use crate::util::{
    launch_fig,
    LaunchArgs,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
enum FigWebsocketMessage {
    DotfilesUpdated,
    #[serde(rename_all = "camelCase")]
    SettingsUpdated {
        settings: serde_json::Map<String, serde_json::Value>,
        #[serde(with = "time::serde::rfc3339::option")]
        updated_at: Option<time::OffsetDateTime>,
    },
    #[serde(rename_all = "camelCase")]
    Event {
        event_name: String,
        payload: Option<serde_json::Value>,
        apps: Option<Vec<String>>,
    },
    #[serde(rename_all = "camelCase")]
    TriggerAutoUpdate,
    #[serde(rename_all = "camelCase")]
    QuitDaemon {
        status: Option<i32>,
    },
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TicketBody {
    ticket: String,
    fly_instance: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RateLimitResponse {
    error: Option<String>,
    timeout: Option<u64>,
}

pub async fn connect_to_fig_websocket() -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
    info!("Connecting to websocket");

    let mut params = vec![
        ("os", std::env::consts::OS.into()),
        ("arch", std::env::consts::ARCH.into()),
        ("cliVersion", env!("CARGO_PKG_VERSION").into()),
    ];

    if let Some(version) = fig_util::manifest::version() {
        params.push(("manifestVersion", version.into()));
    }

    if let Ok(mut device_id) = get_system_id() {
        if let Some(email) = get_email() {
            device_id.push(':');
            device_id.push_str(&email);
        }
        params.push(("deviceId", device_id));
    }

    let ticket_response = match Request::get("/authenticate/ticket")
        .query(&params)
        .auth()
        .send()
        .await?
    {
        resp if resp.status() == StatusCode::TOO_MANY_REQUESTS => {
            if let Ok(rate_limit) = resp.json::<RateLimitResponse>().await {
                if let Some(timeout) = rate_limit.timeout {
                    warn!(?rate_limit, "Timedout");
                    tokio::time::sleep(Duration::from_millis(timeout)).await;
                }
                if let Some(error) = rate_limit.error {
                    bail!(error);
                }
            }
            bail!(StatusCode::TOO_MANY_REQUESTS.as_str());
        },
        resp => resp.handle_fig_response().await?,
    };

    let ticket_body: TicketBody = match ticket_response
        .headers()
        .get("content-type")
        .and_then(|header| header.to_str().ok())
        .and_then(|content_type| content_type.split_once(';'))
        .map(|(v, _)| v)
    {
        Some("application/json") => ticket_response.json().await?,
        _ => TicketBody {
            ticket: ticket_response.text().await?,
            ..Default::default()
        },
    };

    params.push(("ticket", ticket_body.ticket.clone()));

    if let Some(ref fly_instance) = ticket_body.fly_instance {
        params.push(("flyInstance", fly_instance.clone()));
    }

    let url = Url::parse_with_params(ws_host().as_str(), &params)?;

    debug!("Connecting to {url}");

    let (websocket_stream, _) = tokio::time::timeout(Duration::from_secs(30), tokio_tungstenite::connect_async(url))
        .await
        .context("Websocket connection timedout")?
        .context("Failed to connect to websocket")?;

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
                    debug!("message: {text:?}");
                    let websocket_message_result = serde_json::from_str::<FigWebsocketMessage>(text);

                    match websocket_message_result {
                        Ok(websocket_message) => match websocket_message {
                            FigWebsocketMessage::DotfilesUpdated => scheduler.schedule_now(SyncDotfiles),
                            FigWebsocketMessage::SettingsUpdated { settings, updated_at } => {
                                // Write settings to disk
                                let path =
                                    fig_settings::settings::settings_path().context("Could not get settings path")?;

                                info!("Settings updated: Writing settings to disk at {path:?}");

                                let mut settings_file = std::fs::File::create(&path)?;
                                let settings_json = serde_json::to_string_pretty(&settings)?;
                                settings_file.write_all(settings_json.as_bytes())?;

                                if let Some(updated_at) = updated_at {
                                    if let Ok(updated_at) = updated_at.format(&Rfc3339) {
                                        fig_settings::state::set_value("settings.updatedAt", json!(updated_at)).ok();
                                    }
                                }
                            },
                            FigWebsocketMessage::Event {
                                event_name,
                                payload,
                                apps,
                            } => match payload.as_ref().map(serde_json::to_string).transpose() {
                                Err(err) => error!("Could not serialize event payload: {err:?}"),
                                Ok(payload_blob) => {
                                    let hook = new_event_hook(event_name, payload_blob, apps.unwrap_or_default());
                                    send_hook_to_socket(hook).await.ok();
                                },
                            },
                            FigWebsocketMessage::TriggerAutoUpdate => {
                                if !settings::get_bool_or("app.disableAutoupdates", false) {
                                    // trigger forced update. This will QUIT the macOS app, it must be relaunched...
                                    update_command(true).await.ok();

                                    // Sleep for a bit
                                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

                                    // Relaunch the app
                                    launch_fig(LaunchArgs {
                                        print_running: false,
                                        print_launching: false,
                                        wait_for_launch: true,
                                    })
                                    .ok();
                                }
                            },
                            FigWebsocketMessage::QuitDaemon { status } => std::process::exit(status.unwrap_or(0)),
                        },
                        Err(err) => error!("Could not parse json message: {err:?}"),
                    }
                    Ok(())
                },
                Message::Close(close_frame) => match close_frame {
                    Some(close_frame) => {
                        info!("Websocket close frame: {close_frame:?}");
                        Err(eyre!("Websocket close frame: {close_frame:?}"))
                    },
                    None => {
                        info!("Websocket close frame");
                        Err(eyre!("Websocket close frame"))
                    },
                },
                Message::Ping(_) => {
                    debug!("Websocket ping");
                    Ok(())
                },
                Message::Pong(_) => {
                    debug!("Websocket pong");
                    Ok(())
                },
                unknown_message => {
                    debug!("Unknown message: {unknown_message:?}");
                    Ok(())
                },
            },
            Err(err) => {
                error!("Websock next error: {err:?}");
                Err(eyre!("Websock next error: {err:?}"))
            },
        },
        None => {
            info!("Websocket closed");
            Err(eyre!("Websocket closed"))
        },
    }
}
