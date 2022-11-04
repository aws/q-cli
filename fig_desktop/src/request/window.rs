use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{
    FocusAction,
    PositionWindowRequest,
    PositionWindowResponse,
    WindowFocusRequest,
};
use tracing::debug;
use wry::application::dpi::LogicalSize;

use super::{
    RequestResult,
    RequestResultImpl,
};
use crate::event::{
    Event,
    WindowEvent,
};
use crate::figterm::FigtermState;
use crate::platform::PlatformState;
use crate::webview::window::WindowId;
use crate::{
    EventLoopProxy,
    AUTOCOMPLETE_ID,
};

pub async fn position_window(
    request: PositionWindowRequest,
    window_id: WindowId,
    platform_state: &PlatformState,
    figterm_state: &FigtermState,
    proxy: &EventLoopProxy,
) -> RequestResult {
    debug!(?request, %window_id, "Position Window Request");

    if window_id == AUTOCOMPLETE_ID
        && figterm_state
            .most_recent()
            .and_then(|session| session.context.as_ref().map(|context| context.preexec()))
            .unwrap_or(false)
    {
        return RequestResult::error("Cannot position autocomplete window while preexec is active");
    }

    if request.dryrun.unwrap_or(false) {
        match platform_state.get_active_window() {
            Some(_) => {
                // TODO(grant): do something with geometry
                return RequestResult::Ok(Box::new(ServerOriginatedSubMessage::PositionWindowResponse(
                    PositionWindowResponse {
                        is_above: Some(false),
                        is_clipped: Some(false),
                    },
                )));
            },
            None => {
                return RequestResult::Ok(Box::new(ServerOriginatedSubMessage::PositionWindowResponse(
                    PositionWindowResponse {
                        is_above: Some(false),
                        is_clipped: Some(false),
                    },
                )));
            },
        }
    }

    let anchor = request.anchor.as_ref().expect("missing anchor field");
    let autocomplete_padding = 5.0;
    let size = request.size.as_ref().expect("missing size field");

    proxy
        .send_event(Event::WindowEvent {
            window_id,
            window_event: WindowEvent::Batch(vec![
                WindowEvent::UpdateWindowGeometry {
                    position: None,
                    size: Some(LogicalSize::new(size.width.into(), size.height.into())),
                    anchor: Some(LogicalSize::new(
                        anchor.x.into(),
                        (anchor.y + autocomplete_padding).into(),
                    )),
                },
                // Workaround to nonapplicably zero sized windows
                if size.width == 1.0 || size.height == 1.0 {
                    WindowEvent::Hide
                } else {
                    WindowEvent::Show
                },
            ]),
        })
        .unwrap();

    RequestResult::Ok(Box::new(ServerOriginatedSubMessage::PositionWindowResponse(
        PositionWindowResponse {
            is_above: Some(false),
            is_clipped: Some(false),
        },
    )))
}

pub async fn focus(request: WindowFocusRequest, window_id: WindowId, proxy: &EventLoopProxy) -> RequestResult {
    debug!(?request, %window_id, "Window Focus Request");
    match request.r#type() {
        FocusAction::TakeFocus => {
            proxy
                .send_event(Event::WindowEvent {
                    window_id,
                    window_event: WindowEvent::Show,
                })
                .unwrap();
        },
        FocusAction::ReturnFocus => todo!(),
    }

    RequestResult::success()
}
