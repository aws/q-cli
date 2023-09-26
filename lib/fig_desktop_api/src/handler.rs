use std::time::Duration;

use base64::prelude::*;
pub use fig_proto::fig::client_originated_message::Submessage as ClientOriginatedSubMessage;
pub use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{
    AggregateSessionMetricActionRequest,
    ClientOriginatedMessage,
    DebuggerUpdateRequest,
    InsertTextRequest,
    NotificationRequest,
    OnboardingRequest,
    PositionWindowRequest,
    PseudoterminalExecuteRequest,
    PseudoterminalWriteRequest,
    RunProcessRequest,
    ServerOriginatedMessage,
    UpdateApplicationPropertiesRequest,
    UserLogoutRequest,
    WindowFocusRequest,
};
use fig_proto::prost::Message;
use tracing::warn;

use crate::error::Result;
use crate::kv::KVStore;
use crate::requests::{
    self,
    RequestResult,
    RequestResultImpl,
};

pub struct Wrapped<Ctx, Req> {
    pub message_id: i64,
    pub context: Ctx,
    pub request: Req,
}

#[async_trait::async_trait]
pub trait EventHandler {
    type Ctx: KVStore + Sync + Send;

    async fn notification(&self, request: Wrapped<Self::Ctx, NotificationRequest>) -> RequestResult {
        RequestResult::unimplemented(request.request)
    }

    async fn debugger_update(&self, request: Wrapped<Self::Ctx, DebuggerUpdateRequest>) -> RequestResult {
        RequestResult::unimplemented(request.request)
    }

    async fn insert_text(&self, request: Wrapped<Self::Ctx, InsertTextRequest>) -> RequestResult {
        RequestResult::unimplemented(request.request)
    }

    async fn aggregate_session_metric_action(
        &self,
        request: Wrapped<Self::Ctx, AggregateSessionMetricActionRequest>,
    ) -> RequestResult {
        RequestResult::unimplemented(request.request)
    }

    async fn position_window(&self, request: Wrapped<Self::Ctx, PositionWindowRequest>) -> RequestResult {
        RequestResult::unimplemented(request.request)
    }

    async fn window_focus(&self, request: Wrapped<Self::Ctx, WindowFocusRequest>) -> RequestResult {
        RequestResult::unimplemented(request.request)
    }

    async fn onboarding(&self, request: Wrapped<Self::Ctx, OnboardingRequest>) -> RequestResult {
        RequestResult::unimplemented(request.request)
    }

    async fn run_process(&self, request: Wrapped<Self::Ctx, RunProcessRequest>) -> RequestResult {
        RequestResult::unimplemented(request.request)
    }

    async fn pseudoterminal_execute(&self, request: Wrapped<Self::Ctx, PseudoterminalExecuteRequest>) -> RequestResult {
        RequestResult::unimplemented(request.request)
    }

    async fn pseudoterminal_write(&self, request: Wrapped<Self::Ctx, PseudoterminalWriteRequest>) -> RequestResult {
        RequestResult::unimplemented(request.request)
    }

    async fn update_application_properties(
        &self,
        request: Wrapped<Self::Ctx, UpdateApplicationPropertiesRequest>,
    ) -> RequestResult {
        RequestResult::unimplemented(request.request)
    }

    async fn user_logout(&self, request: Wrapped<Self::Ctx, UserLogoutRequest>) -> RequestResult {
        RequestResult::unimplemented(request.request)
    }
}

pub fn request_from_b64(request_b64: &str) -> Result<ClientOriginatedMessage> {
    let data = BASE64_STANDARD.decode(request_b64)?;
    Ok(ClientOriginatedMessage::decode(data.as_slice())?)
}

pub fn response_to_b64(response_message: ServerOriginatedMessage) -> String {
    BASE64_STANDARD.encode(response_message.encode_to_vec())
}

pub async fn api_request<Ctx: KVStore, E: EventHandler<Ctx = Ctx> + Sync>(
    event_handler: E,
    ctx: Ctx,
    request: ClientOriginatedMessage,
) -> Result<ServerOriginatedMessage> {
    let request_id = match request.id {
        Some(request_id) => request_id,
        None => return Err(crate::error::Error::NoMessageId),
    };

    let response = match tokio::time::timeout(
        Duration::from_secs(30),
        handle_request(event_handler, ctx, request_id, request),
    )
    .await
    {
        Ok(response) => response,
        Err(_) => return Err(crate::error::Error::Timeout),
    };

    Ok(ServerOriginatedMessage {
        id: Some(request_id),
        submessage: Some(match response {
            Ok(msg) => *msg,
            Err(msg) => ServerOriginatedSubMessage::Error(msg.to_string()),
        }),
    })
}

async fn handle_request<Ctx: KVStore, E: EventHandler<Ctx = Ctx> + Sync>(
    event_handler: E,
    ctx: Ctx,
    message_id: i64,
    message: ClientOriginatedMessage,
) -> RequestResult {
    macro_rules! request {
        ($request:expr) => {
            Wrapped {
                message_id,
                context: ctx,
                request: $request,
            }
        };
    }

    match message.submessage {
        Some(submessage) => {
            use requests::*;
            use ClientOriginatedSubMessage::*;

            match submessage {
                // debug
                DebuggerUpdateRequest(request) => event_handler.debugger_update(request!(request)).await,
                // figterm
                InsertTextRequest(request) => event_handler.insert_text(request!(request)).await,
                // fs
                ReadFileRequest(request) => fs::read_file(request).await,
                WriteFileRequest(request) => fs::write_file(request).await,
                AppendToFileRequest(request) => fs::append_to_file(request).await,
                DestinationOfSymbolicLinkRequest(request) => requests::fs::destination_of_symbolic_link(request).await,
                ContentsOfDirectoryRequest(request) => fs::contents_of_directory(request).await,
                CreateDirectoryRequest(request) => fs::create_directory_request(request).await,
                // notifications
                NotificationRequest(request) => event_handler.notification(request!(request)).await,
                // process
                RunProcessRequest(request) => event_handler.run_process(request!(request)).await,
                PseudoterminalExecuteRequest(request) => event_handler.pseudoterminal_execute(request!(request)).await,
                PseudoterminalWriteRequest(request) => event_handler.pseudoterminal_write(request!(request)).await,
                // properties
                UpdateApplicationPropertiesRequest(request) => {
                    event_handler.update_application_properties(request!(request)).await
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
                TelemetryIdentifyRequest(request) => telemetry::handle_identify_request(request).await,
                TelemetryTrackRequest(request) => telemetry::handle_track_request(request).await,
                TelemetryPageRequest(request) => telemetry::handle_page_request(request).await,
                AggregateSessionMetricActionRequest(request) => {
                    event_handler.aggregate_session_metric_action(request!(request)).await
                },
                // window
                PositionWindowRequest(request) => event_handler.position_window(request!(request)).await,
                WindowFocusRequest(request) => event_handler.window_focus(request!(request)).await,
                // onboarding
                OnboardingRequest(request) => event_handler.onboarding(request!(request)).await,
                // install
                InstallRequest(request) => install::install(request).await,
                // history
                HistoryQueryRequest(request) => history::query(request).await,
                // auth
                AuthStatusRequest(request) => auth::status(request).await,
                AuthBuilderIdInitRequest(request) => auth::builder_id_init(request, &ctx).await,
                AuthBuilderIdPollRequest(request) => auth::builder_id_poll(request, &ctx).await,
                // other
                OpenInExternalApplicationRequest(request) => other::open_in_external_application(request).await,
                // deprecated
                GetConfigPropertyRequest(request) => RequestResult::deprecated(request),
                UpdateConfigPropertyRequest(request) => RequestResult::deprecated(request),
                PseudoterminalRestartRequest(request) => RequestResult::deprecated(request),
                TerminalSessionInfoRequest(request) => RequestResult::deprecated(request),
                ApplicationUpdateStatusRequest(request) => RequestResult::deprecated(request),
                MacosInputMethodRequest(request) => RequestResult::deprecated(request),
                UserLogoutRequest(request) => event_handler.user_logout(request!(request)).await,
                UpdateApplicationRequest(request) => update::update_application(request).await,
                CheckForUpdatesRequest(request) => update::check_for_updates(request).await,
            }
        },
        None => {
            warn!("Missing submessage: {message:?}");
            RequestResult::error("Missing submessage")
        },
    }
}
