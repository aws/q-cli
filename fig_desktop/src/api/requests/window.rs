use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{
    FocusAction,
    PositionWindowRequest,
    PositionWindowResponse,
    WindowFocusRequest,
};
use tracing::debug;
use wry::application::dpi::{
    LogicalPosition,
    LogicalSize,
};

use super::{
    RequestResult,
    RequestResultImpl,
};
use crate::event::{
    Event,
    WindowEvent,
};
use crate::platform::PlatformState;
use crate::webview::window::WindowId;
use crate::EventLoopProxy;

pub async fn position_window(
    request: PositionWindowRequest,
    window_id: WindowId,
    platform_state: &PlatformState,
    proxy: &EventLoopProxy,
) -> RequestResult {
    debug!(?request, %window_id, "Position Window Request");
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

    let size = request.size.as_ref().expect("Missing size field");
    let anchor = request.anchor.expect("Missing anchor field");

    proxy
        .send_event(Event::WindowEvent {
            window_id: window_id.clone(),
            window_event: WindowEvent::Resize {
                size: LogicalSize::new(size.width.into(), size.height.into()),
            },
        })
        .unwrap();

    proxy
        .send_event(Event::WindowEvent {
            window_id: window_id.clone(),
            window_event: WindowEvent::Reanchor {
                position: LogicalPosition::new(anchor.x.into(), anchor.y.into()),
            },
        })
        .unwrap();

    // NOTE(mia): this code never restores the window on linux

    // Workaround to nonapplicably zero sized windows
    match size.width == 1.0 || size.height == 1.0 {
        true => {
            proxy
                .send_event(Event::WindowEvent {
                    window_id,
                    window_event: WindowEvent::Hide,
                })
                .unwrap();
        },
        false => {
            proxy
                .send_event(Event::WindowEvent {
                    window_id,
                    window_event: WindowEvent::Show,
                })
                .unwrap();
        },
    }

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
