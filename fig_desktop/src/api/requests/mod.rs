mod debug;
mod defaults;
mod figterm;
mod fs;
mod install;
mod notifications;
mod onboarding;
mod other;
mod process;
mod properties;
mod settings;
mod state;
mod telemetry;
mod window;

use std::time::Duration;

use anyhow::Result;
use fig_proto::fig::client_originated_message::Submessage as ClientOriginatedSubMessage;
use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{
    ClientOriginatedMessage,
    ServerOriginatedMessage,
};
use fig_proto::prost::Message;
use fig_proto::ReflectMessage;
use tracing::{
    trace,
    warn,
};

use crate::event::{
    Event,
    WindowEvent,
};
use crate::figterm::FigtermState;
use crate::native::NativeState;
use crate::notification::NotificationsState;
use crate::utils::truncate_string;
use crate::webview::window::WindowId;
use crate::{
    DebugState,
    EventLoopProxy,
    InterceptState,
    FIG_PROTO_MESSAGE_RECEIVED,
};

static FIG_GLOBAL_ERROR_OCCURRED: &str = "FigGlobalErrorOccurred";

type RequestResult = Result<Box<ServerOriginatedSubMessage>>;

trait RequestResultImpl {
    fn success() -> Self;
    fn error(msg: impl Into<String>) -> Self;
    fn deprecated(message: impl ReflectMessage) -> Self;
}

impl RequestResultImpl for RequestResult {
    fn success() -> Self {
        RequestResult::Ok(Box::new(ServerOriginatedSubMessage::Success(true)))
    }

    fn error(msg: impl Into<String>) -> Self {
        RequestResult::Ok(Box::new(ServerOriginatedSubMessage::Error(msg.into())))
    }

    fn deprecated(message: impl ReflectMessage) -> Self {
        RequestResult::error(format!("{} is deprecated", message.descriptor().name()))
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn api_request(
    window_id: WindowId,
    client_originated_message_b64: String,
    debug_state: &DebugState,
    figterm_state: &FigtermState,
    intercept_state: &InterceptState,
    notifications_state: &NotificationsState,
    native_state: &NativeState,
    proxy: &EventLoopProxy,
) {
    let data = match base64::decode(client_originated_message_b64) {
        Ok(data) => data,
        Err(err) => {
            warn!("Failed to decode base64 from {window_id}: {err}");
            proxy
                .send_event(Event::WindowEvent {
                    window_id,
                    window_event: WindowEvent::Emit {
                        event: FIG_GLOBAL_ERROR_OCCURRED.into(),
                        payload: format!("Failed to decode base64: {err}"),
                    },
                })
                .unwrap();
            return;
        },
    };

    let request = match ClientOriginatedMessage::decode(data.as_slice()) {
        Ok(request) => request,
        Err(err) => {
            warn!("Failed to decode proto from {window_id}: {err}");
            proxy
                .send_event(Event::WindowEvent {
                    window_id,
                    window_event: WindowEvent::Emit {
                        event: FIG_GLOBAL_ERROR_OCCURRED.into(),
                        payload: format!("Failed to decode proto: {err}"),
                    },
                })
                .unwrap();
            return;
        },
    };

    trace!(?request, "Received request from {window_id}");

    let request_id = match request.id {
        Some(request_id) => request_id,
        None => {
            warn!("No message_id provided from {window_id}");
            proxy
                .send_event(Event::WindowEvent {
                    window_id,
                    window_event: WindowEvent::Emit {
                        event: FIG_GLOBAL_ERROR_OCCURRED.into(),
                        payload: "No message_id provided".into(),
                    },
                })
                .unwrap();
            return;
        },
    };

    let response = match tokio::time::timeout(
        Duration::from_secs(30),
        handle_request(
            request,
            request_id,
            &window_id,
            debug_state,
            figterm_state,
            intercept_state,
            notifications_state,
            native_state,
            proxy,
        ),
    )
    .await
    {
        Ok(response) => response,
        Err(_) => {
            warn!(%window_id, "Fig API request timedout after 30 seconds");
            RequestResult::error("Request timedout after 30 seconds")
        },
    };

    trace!(?response, "Sending response to {window_id}");

    let message = ServerOriginatedMessage {
        id: Some(request_id),
        submessage: Some(match response {
            Ok(msg) => *msg,
            Err(msg) => {
                warn!("Send error response for {window_id}: {msg}");
                ServerOriginatedSubMessage::Error(msg.to_string())
            },
        }),
    };

    proxy
        .send_event(Event::WindowEvent {
            window_id,
            window_event: WindowEvent::Emit {
                event: FIG_PROTO_MESSAGE_RECEIVED.into(),
                payload: base64::encode(message.encode_to_vec()),
            },
        })
        .unwrap();
}

#[allow(clippy::too_many_arguments)]
async fn handle_request(
    message: ClientOriginatedMessage,
    message_id: i64,
    window_id: &WindowId,
    debug_state: &DebugState,
    figterm_state: &FigtermState,
    intercept_state: &InterceptState,
    notifications_state: &NotificationsState,
    native_state: &NativeState,
    proxy: &EventLoopProxy,
) -> RequestResult {
    match message.submessage {
        Some(submessage) => {
            use ClientOriginatedSubMessage::*;

            match submessage {
                // debug
                DebuggerUpdateRequest(request) => debug::update(request, debug_state).await,
                // figterm
                InsertTextRequest(request) => figterm::insert_text(request, figterm_state).await,
                // fs
                ReadFileRequest(request) => fs::read_file(request).await,
                WriteFileRequest(request) => fs::write_file(request).await,
                AppendToFileRequest(request) => fs::append_to_file(request).await,
                DestinationOfSymbolicLinkRequest(request) => fs::destination_of_symbolic_link(request).await,
                ContentsOfDirectoryRequest(request) => fs::contents_of_directory(request).await,
                CreateDirectoryRequest(request) => fs::create_directory_request(request).await,
                // notifications
                NotificationRequest(request) => {
                    notifications::handle_request(request, window_id.clone(), message_id, notifications_state).await
                },
                // process
                RunProcessRequest(request) => process::run(request, figterm_state).await,
                PseudoterminalExecuteRequest(request) => process::execute(request, figterm_state).await,
                PseudoterminalWriteRequest(_deprecated) => process::write().await,
                // properties
                UpdateApplicationPropertiesRequest(request) => {
                    properties::update(request, figterm_state, intercept_state)
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
                TelemetryPageRequest(request) => telemetry::handle_page_request(request).await,
                AggregateSessionMetricActionRequest(request) => {
                    telemetry::handle_aggregate_session_metric_action_request(request, figterm_state)
                },
                // window
                PositionWindowRequest(request) => {
                    window::position_window(request, window_id.clone(), native_state, proxy).await
                },
                WindowFocusRequest(request) => window::focus(request, window_id.clone(), proxy).await,
                // onboarding
                OnboardingRequest(request) => onboarding::onboarding(request, proxy).await,
                // install
                InstallRequest(request) => install::install(request).await,
                // other
                OpenInExternalApplicationRequest(request) => other::open_in_external_application(request).await,
                // depercated
                GetConfigPropertyRequest(request) => RequestResult::deprecated(request),
                UpdateConfigPropertyRequest(request) => RequestResult::deprecated(request),
                PseudoterminalRestartRequest(request) => RequestResult::deprecated(request),
                TerminalSessionInfoRequest(request) => RequestResult::deprecated(request),
                ApplicationUpdateStatusRequest(request) => RequestResult::deprecated(request),
                MacosInputMethodRequest(request) => RequestResult::deprecated(request),
            }
        },
        None => {
            let truncated = truncate_string(format!("{message:?}"), 150);
            warn!("Missing submessage: {truncated}");
            RequestResult::error("Missing submessage")
        },
    }
}
