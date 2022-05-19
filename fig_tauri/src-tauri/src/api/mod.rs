mod debug;
mod defaults;
mod figterm;
mod fs;
mod notifications;
mod process;
mod properties;
mod settings;
mod state;
mod telemetry;
mod window;

use anyhow::Result;
use bytes::BytesMut;
use fig_proto::fig::client_originated_message::Submessage as ClientOriginatedSubMessage;
use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{
    ClientOriginatedMessage,
    ServerOriginatedMessage,
};
use fig_proto::prost::Message;
use tracing::{
    debug,
    warn,
};
use wry::application::event_loop::EventLoopProxy;

use crate::utils::truncate_string;
use crate::window::FigWindowEvent;
use crate::{
    FigEvent,
    FigId,
    GlobalState,
    FIG_PROTO_MESSAGE_RECIEVED,
};

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

pub async fn api_request(
    fig_id: FigId,
    client_originated_message_b64: String,
    global_state: &GlobalState,
    proxy: &EventLoopProxy<FigEvent>,
) {
    let data = base64::decode(client_originated_message_b64).unwrap();

    let message = match ClientOriginatedMessage::decode(data.as_slice()) {
        Ok(message) => message,
        Err(err) => {
            warn!("Failed to decode request: {err}");
            proxy
                .send_event(FigEvent::WindowEvent {
                    fig_id,
                    window_event: FigWindowEvent::Emit {
                        event: FIG_GLOBAL_ERROR_OCCURRED.into(),
                        payload: "Decode error".into(),
                    },
                })
                .unwrap();
            return;
        },
    };

    debug!("{message:?}");

    // TODO: return error
    let message_id = message.id.unwrap();

    let response = match message.submessage {
        None => {
            let truncated = truncate_string(format!("{message:?}"), 150);
            warn!("Missing submessage: {}", truncated);
            RequestResult::error(format!("Missing submessage {truncated}"))
        },
        Some(submessage) => {
            use ClientOriginatedSubMessage::*;

            match submessage {
                // debug
                DebuggerUpdateRequest(request) => debug::update(request, &global_state.debug_state).await,
                // figterm
                InsertTextRequest(request) => figterm::insert_text(request, &global_state.figterm_state).await,
                // fs
                ReadFileRequest(request) => fs::read_file(request).await,
                WriteFileRequest(request) => fs::write_file(request).await,
                AppendToFileRequest(request) => fs::append_to_file(request).await,
                DestinationOfSymbolicLinkRequest(request) => fs::destination_of_symbolic_link(request).await,
                ContentsOfDirectoryRequest(request) => fs::contents_of_directory(request).await,
                // notifications
                NotificationRequest(request) => {
                    notifications::handle_request(
                        request,
                        fig_id.clone(),
                        message_id,
                        &global_state.notifications_state,
                    )
                    .await
                },
                // process
                RunProcessRequest(request) => process::run(request).await,
                PseudoterminalExecuteRequest(request) => process::execute(request).await,
                PseudoterminalWriteRequest(_deprecated) => process::write().await,
                // properties
                UpdateApplicationPropertiesRequest(request) => {
                    properties::update(request, &global_state.figterm_state, &global_state.intercept_state).await
                },
                // state
                GetLocalStateRequest(request) => state::get(request).await,
                UpdateLocalStateRequest(request) => state::update(request).await,
                // settings
                GetSettingsPropertyRequest(request) => settings::get(request).await,
                UpdateSettingsPropertyRequest(request) => settings::update(request).await,
                // defaults
                GetDefaultsPropertyRequest(request) => defaults::get(request).await,
                UpdateDefaultsPropertyRequest(request) => defaults::update(request).await,
                // telemetry
                TelemetryAliasRequest(request) => telemetry::handle_alias_request(request).await,
                TelemetryIdentifyRequest(request) => telemetry::handle_identify_request(request).await,
                TelemetryTrackRequest(request) => telemetry::handle_track_request(request).await,
                // window
                PositionWindowRequest(request) => window::position_window(request, fig_id.clone(), proxy).await,
                unknown => {
                    warn!("Missing handler: {unknown:?}");
                    RequestResult::error(format!("Unknown submessage {unknown:?}"))
                },
            }
        },
    };

    debug!("response: {response:?}");

    let message = ServerOriginatedMessage {
        id: message.id,
        submessage: Some(match response {
            Ok(msg) => *msg,
            Err(msg) => {
                warn!("Send error response: {}", msg);
                ServerOriginatedSubMessage::Error(msg.to_string())
            },
        }),
    };

    let mut encoded = BytesMut::new();
    if message.encode(&mut encoded).is_err() {
        proxy
            .send_event(FigEvent::WindowEvent {
                fig_id,
                window_event: FigWindowEvent::Emit {
                    event: FIG_GLOBAL_ERROR_OCCURRED.into(),
                    payload: "Encode error".into(),
                },
            })
            .unwrap();
    } else {
        proxy
            .send_event(FigEvent::WindowEvent {
                fig_id,
                window_event: FigWindowEvent::Emit {
                    event: FIG_PROTO_MESSAGE_RECIEVED.into(),
                    payload: base64::encode(encoded),
                },
            })
            .unwrap();
    }
}
