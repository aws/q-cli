use std::io::Write;
use std::time::Duration;

use eyre::{
    bail,
    eyre,
    Result,
    WrapErr,
};
use fig_api_client::settings::ensure_telemetry;
use fig_install::UpdateOptions;
use fig_ipc::local::send_hook_to_socket;
use fig_proto::hooks::new_event_hook;
use fig_request::auth::get_email;
use fig_request::reqwest::StatusCode;
use fig_request::reqwest_client::client_config;
use fig_request::Request;
use fig_settings::api::ws_host;
use fig_settings::settings;
use fig_util::consts::FIG_SCRIPTS_SCHEMA_VERSION;
use fig_util::directories;
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
    TriggerAutoUpdate {
        #[serde(default)]
        ignore_rollout: bool,
    },
    #[serde(rename_all = "camelCase")]
    QuitDaemon {
        status: Option<i32>,
    },
    // Scripts ////////////////////////////////////////////////////////////
    /// Update a single script in the local cache given a script identifier
    #[serde(rename_all = "camelCase")]
    UpdateScript {
        script: ScriptIdentifier,
    },
    /// Delete a script with the given identifier
    #[serde(rename_all = "camelCase")]
    DeleteScript {
        script: ScriptIdentifier,
    },
    /// Rename a script by first deleting then retrieving the new one
    #[serde(rename_all = "camelCase")]
    RenameScript {
        old: ScriptIdentifier,
        new: ScriptIdentifier,
    },
    /// Update the local scripts cache
    #[serde(rename_all = "camelCase")]
    UpdateAllScripts,
    /// Refresh local commandline tools cache
    #[serde(rename_all = "camelCase")]
    UpdateAllCommandlineTools,
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

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ScriptIdentifier {
    namespace: String,
    name: String,
}

pub async fn connect_to_fig_websocket() -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
    info!("Connecting to websocket");

    let mut params = vec![
        ("os", std::env::consts::OS.into()),
        ("arch", std::env::consts::ARCH.into()),
        ("cliVersion", env!("CARGO_PKG_VERSION").into()),
    ];

    params.push(("manifestVersion", env!("CARGO_PKG_VERSION").to_string()));

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

    let (websocket_stream, _) = tokio_tungstenite::connect_async_tls_with_config(
        url,
        None,
        Some(tokio_tungstenite::Connector::Rustls(client_config(true))),
    )
    .await
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
                        Ok(websocket_message) => match process_message(websocket_message, scheduler).await {
                            Ok(_) => (),
                            Err(err) => error!(%err, "Could not process message"),
                        },
                        Err(err) => error!(%err, "Could not parse json message"),
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

async fn process_message(message: FigWebsocketMessage, scheduler: &mut Scheduler) -> Result<()> {
    match message {
        FigWebsocketMessage::DotfilesUpdated => {
            scheduler.schedule_now(SyncDotfiles);
            Ok(())
        },
        FigWebsocketMessage::SettingsUpdated {
            mut settings,
            updated_at,
        } => {
            if let Err(err) = ensure_telemetry(&mut settings).await {
                error!(?err, "Failed to ensure telemetry is respected");
            }

            // Write settings to disk
            let path = directories::settings_path().context("Could not get settings path")?;

            info!(?path, "Settings updated: Writing settings to disk");

            let mut settings_file = std::fs::File::create(&path)?;
            let settings_json = serde_json::to_string_pretty(&settings)?;
            settings_file.write_all(settings_json.as_bytes())?;

            if let Some(updated_at) = updated_at {
                if let Ok(updated_at) = updated_at.format(&Rfc3339) {
                    fig_settings::state::set_value("settings.updatedAt", json!(updated_at)).ok();
                }
            }

            Ok(())
        },
        FigWebsocketMessage::Event {
            event_name,
            payload,
            apps,
        } => {
            match payload.as_ref().map(serde_json::to_string).transpose() {
                Err(err) => error!(%err, "Could not serialize event payload"),
                Ok(payload_blob) => {
                    let hook = new_event_hook(event_name, payload_blob, apps.unwrap_or_default());
                    send_hook_to_socket(hook).await.ok();
                },
            };
            Ok(())
        },
        FigWebsocketMessage::TriggerAutoUpdate { ignore_rollout } => {
            if !settings::get_bool_or("app.disableAutoupdates", false) {
                fig_install::update(None, UpdateOptions {
                    ignore_rollout,
                    interactive: false,
                    relaunch_dashboard: false,
                })
                .await
                .ok();
            }

            Ok(())
        },
        FigWebsocketMessage::QuitDaemon { status } => std::process::exit(status.unwrap_or(0)),
        FigWebsocketMessage::UpdateScript { script } => {
            let scripts_cache_dir = directories::scripts_cache_dir()?;
            tokio::fs::create_dir_all(&scripts_cache_dir).await?;

            let file_name = scripts_cache_dir.join(format!("{}.{}.json", script.namespace, script.name));
            tokio::fs::remove_file(&file_name).await.ok();

            match fig_api_client::scripts::script(&script.namespace, &script.name, FIG_SCRIPTS_SCHEMA_VERSION).await {
                Ok(script) => {
                    tokio::fs::write(&file_name, serde_json::to_string_pretty(&script)?.as_bytes()).await?;
                },
                Err(err) => error!(%err, "Failed to retrieve script"),
            }

            Ok(())
        },
        FigWebsocketMessage::DeleteScript { script } => {
            tokio::fs::remove_file(
                directories::scripts_cache_dir()?.join(format!("{}.{}.json", script.namespace, script.name)),
            )
            .await
            .ok();

            Ok(())
        },
        FigWebsocketMessage::RenameScript { old, new } => {
            let scripts_cache_dir = directories::scripts_cache_dir()?;
            tokio::fs::create_dir_all(&scripts_cache_dir).await?;

            tokio::fs::remove_file(scripts_cache_dir.join(format!("{}.{}.json", old.namespace, old.name)))
                .await
                .ok();

            if let Ok(script) =
                fig_api_client::scripts::script(&new.namespace, &new.name, FIG_SCRIPTS_SCHEMA_VERSION).await
            {
                tokio::fs::write(
                    scripts_cache_dir.join(format!("{}.{}.json", new.namespace, new.name)),
                    serde_json::to_string_pretty(&script)?.as_bytes(),
                )
                .await?;
            }

            Ok(())
        },
        FigWebsocketMessage::UpdateAllScripts => {
            if let Err(err) = fig_api_client::scripts::sync_scripts().await {
                error!(?err, "Failed to update scripts");
            }

            Ok(())
        },
        FigWebsocketMessage::UpdateAllCommandlineTools => {
            fig_api_client::commandline_tool::fetch_and_cache_all_command_line_tools()
                .await
                .ok();
            // TODO: refresh cache
            Ok(())
        },
    }
}
