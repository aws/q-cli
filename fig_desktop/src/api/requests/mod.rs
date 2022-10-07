mod debug;
mod figterm;
mod notifications;
mod onboarding;
mod process;
mod properties;
mod telemetry;
mod window;

use std::marker::PhantomData;

use fig_desktop_api::handler::Wrapped;
pub use fig_desktop_api::requests::{
    RequestResult,
    RequestResultImpl,
};
use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{
    AggregateSessionMetricActionRequest,
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
    WindowFocusRequest,
};
use tracing::{
    trace,
    warn,
};

use crate::event::{
    Event,
    WindowEvent,
};
use crate::figterm::FigtermState;
use crate::notification::NotificationsState;
use crate::platform::PlatformState;
use crate::webview::window::WindowId;
use crate::{
    DebugState,
    EventLoopProxy,
    InterceptState,
    FIG_PROTO_MESSAGE_RECEIVED,
};

static FIG_GLOBAL_ERROR_OCCURRED: &str = "FigGlobalErrorOccurred";

struct Context<'a> {
    debug_state: &'a DebugState,
    figterm_state: &'a FigtermState,
    intercept_state: &'a InterceptState,
    notifications_state: &'a NotificationsState,
    platform_state: &'a PlatformState,
    proxy: &'a EventLoopProxy,
    window_id: &'a WindowId,
}

#[derive(Default)]
struct EventHandler<'a> {
    _lifetime: PhantomData<&'a ()>,
}

#[async_trait::async_trait]
impl<'a> fig_desktop_api::handler::EventHandler for EventHandler<'a> {
    type Ctx = Context<'a>;

    async fn notification(&self, request: Wrapped<Self::Ctx, NotificationRequest>) -> RequestResult {
        notifications::handle_request(
            request.request,
            request.context.window_id.clone(),
            request.message_id,
            request.context.notifications_state,
        )
        .await
    }

    async fn debugger_update(&self, request: Wrapped<Self::Ctx, DebuggerUpdateRequest>) -> RequestResult {
        debug::update(request.request, request.context.debug_state).await
    }

    async fn insert_text(&self, request: Wrapped<Self::Ctx, InsertTextRequest>) -> RequestResult {
        figterm::insert_text(request.request, request.context.figterm_state).await
    }

    async fn aggregate_session_metric_action(
        &self,
        request: Wrapped<Self::Ctx, AggregateSessionMetricActionRequest>,
    ) -> RequestResult {
        telemetry::handle_aggregate_session_metric_action_request(request.request, request.context.figterm_state)
    }

    async fn position_window(&self, request: Wrapped<Self::Ctx, PositionWindowRequest>) -> RequestResult {
        window::position_window(
            request.request,
            request.context.window_id.clone(),
            request.context.platform_state,
            request.context.proxy,
        )
        .await
    }

    async fn window_focus(&self, request: Wrapped<Self::Ctx, WindowFocusRequest>) -> RequestResult {
        window::focus(
            request.request,
            request.context.window_id.clone(),
            request.context.proxy,
        )
        .await
    }

    async fn onboarding(&self, request: Wrapped<Self::Ctx, OnboardingRequest>) -> RequestResult {
        onboarding::onboarding(request.request, request.context.proxy).await
    }

    async fn run_process(&self, request: Wrapped<Self::Ctx, RunProcessRequest>) -> RequestResult {
        process::run(request.request, request.context.figterm_state).await
    }

    async fn pseudoterminal_execute(&self, request: Wrapped<Self::Ctx, PseudoterminalExecuteRequest>) -> RequestResult {
        process::execute(request.request, request.context.figterm_state).await
    }

    async fn pseudoterminal_write(&self, _request: Wrapped<Self::Ctx, PseudoterminalWriteRequest>) -> RequestResult {
        process::write().await
    }

    async fn update_application_properties(
        &self,
        request: Wrapped<Self::Ctx, UpdateApplicationPropertiesRequest>,
    ) -> RequestResult {
        properties::update(
            request.request,
            request.context.figterm_state,
            request.context.intercept_state,
        )
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
    platform_state: &PlatformState,
    proxy: &EventLoopProxy,
) {
    let response = match fig_desktop_api::handler::request_from_b64(&client_originated_message_b64) {
        Ok(request) => {
            let id = request.id;
            trace!(?request, %window_id, "Received request");
            match fig_desktop_api::handler::api_request(
                EventHandler::default(),
                Context {
                    debug_state,
                    figterm_state,
                    intercept_state,
                    notifications_state,
                    platform_state,
                    proxy,
                    window_id: &window_id,
                },
                request,
            )
            .await
            {
                Ok(response) => response,
                Err(err) => {
                    warn!(?err, ?id, "Error handling request");
                    ServerOriginatedMessage {
                        id,
                        submessage: Some(ServerOriginatedSubMessage::Error(err.to_string())),
                    }
                },
            }
        },
        Err(err) => {
            warn!(?err, "Error decoding message");
            ServerOriginatedMessage {
                id: None,
                submessage: Some(ServerOriginatedSubMessage::Error(err.to_string())),
            }
        },
    };

    proxy
        .send_event(Event::WindowEvent {
            window_id,
            window_event: WindowEvent::Emit {
                event: match response.id {
                    Some(_) => FIG_PROTO_MESSAGE_RECEIVED.into(),
                    None => FIG_GLOBAL_ERROR_OCCURRED.into(),
                },
                payload: fig_desktop_api::handler::response_to_b64(response),
            },
        })
        .unwrap();
}
