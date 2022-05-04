mod debug;
mod figterm;
mod fs;
mod notifications;
mod process;
mod properties;
mod settings;
mod telemetry;
mod window;

use std::sync::Arc;

use anyhow::Result;
use bytes::BytesMut;
use fig_proto::fig::client_originated_message::Submessage as ClientOriginatedSubMessage;
use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::{
    fig::{ClientOriginatedMessage, ServerOriginatedMessage},
    prost::Message,
};
use tauri::{State, Window};
use tracing::warn;

use crate::figterm::FigtermState;
use crate::utils::truncate_string;
use crate::window::WindowState;
use crate::{DebugState, InterceptState, NotificationsState, FIG_PROTO_MESSAGE_RECIEVED};

const FIG_GLOBAL_ERROR_OCCURRED: &str = "FigGlobalErrorOccurred";

type RequestResult = Result<Box<ServerOriginatedSubMessage>>;

trait RequestResultImpl {
    fn success() -> Self;
    fn error(msg: impl Into<String>) -> Self;
}

impl RequestResultImpl for RequestResult {
    fn success() -> Self {
        RequestResult::Ok(Box::new(ServerOriginatedSubMessage::Success(true)))
    }

    fn error(msg: impl Into<String>) -> Self {
        RequestResult::Ok(Box::new(ServerOriginatedSubMessage::Error(msg.into())))
    }
}

#[tauri::command]
pub async fn handle_api_request(
    window: Window,
    client_originated_message_b64: String,
    debug_state: State<'_, Arc<DebugState>>,
    figterm_state: State<'_, Arc<FigtermState>>,
    intercept_state: State<'_, InterceptState>,
    notification_state: State<'_, Arc<NotificationsState>>,
    window_state: State<'_, Arc<WindowState>>,
) -> Result<(), tauri::Error> {
    let data = base64::decode(client_originated_message_b64).unwrap();

    let message = match ClientOriginatedMessage::decode(data.as_slice()) {
        Ok(message) => message,
        Err(_) => {
            window.emit(FIG_GLOBAL_ERROR_OCCURRED, "Decode error")?;
            return Ok(());
        }
    };

    // TODO: return error
    let message_id = message.id.unwrap();

    let response = match message.submessage {
        None => {
            let truncated = truncate_string(format!("{message:?}"), 150);
            warn!("Missing submessage: {}", truncated);
            RequestResult::error(format!("Missing submessage {truncated}"))
        }
        Some(submessage) => {
            use ClientOriginatedSubMessage::*;

            match submessage {
                // debug
                DebuggerUpdateRequest(request) => debug::update(request, &debug_state).await,
                // figterm
                InsertTextRequest(request) => figterm::insert_text(request, &figterm_state).await,
                // fs
                ReadFileRequest(request) => fs::read_file(request).await,
                WriteFileRequest(request) => fs::write_file(request).await,
                AppendToFileRequest(request) => fs::append_to_file(request).await,
                DestinationOfSymbolicLinkRequest(request) => {
                    fs::destination_of_symbolic_link(request).await
                }
                ContentsOfDirectoryRequest(request) => fs::contents_of_directory(request).await,
                // notifications
                NotificationRequest(request) => {
                    notifications::handle_request(request, message_id, &notification_state).await
                }
                // process
                RunProcessRequest(request) => process::run(request).await,
                PseudoterminalExecuteRequest(request) => process::execute(request).await,
                PseudoterminalWriteRequest(_deprecated) => process::write().await,
                // properties
                UpdateApplicationPropertiesRequest(request) => {
                    properties::update(request, &figterm_state, &intercept_state).await
                }
                // settings
                GetSettingsPropertyRequest(request) => settings::get(request).await,
                UpdateSettingsPropertyRequest(request) => settings::update(request).await,
                // telemetry
                TelemetryAliasRequest(request) => telemetry::handle_alias_request(request).await,
                TelemetryIdentifyRequest(request) => {
                    telemetry::handle_identify_request(request).await
                }
                TelemetryTrackRequest(request) => telemetry::handle_track_request(request).await,
                // window
                PositionWindowRequest(request) => {
                    window::position_window(request, &window_state).await
                }
                unknown => {
                    warn!("Missing handler: {unknown:?}");
                    RequestResult::error(format!("Unknown submessage {unknown:?}"))
                }
            }
        }
    };

    let message = ServerOriginatedMessage {
        id: message.id,
        submessage: Some(match response {
            Ok(msg) => *msg,
            Err(msg) => {
                warn!("Send error response: {}", msg);
                ServerOriginatedSubMessage::Error(msg.to_string())
            }
        }),
    };

    let mut encoded = BytesMut::new();
    if message.encode(&mut encoded).is_err() {
        window.emit(FIG_GLOBAL_ERROR_OCCURRED, "Encode error")?;
        return Ok(());
    };

    window.emit(FIG_PROTO_MESSAGE_RECIEVED, base64::encode(encoded))
}
