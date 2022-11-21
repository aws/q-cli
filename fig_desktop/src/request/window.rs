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
use crate::webview::window::WindowId;
use crate::{
    EventLoopProxy,
    AUTOCOMPLETE_ID,
};

pub async fn position_window(
    request: PositionWindowRequest,
    window_id: WindowId,
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

    let dry_run = request.dryrun.unwrap_or(false);
    let anchor = request.anchor.as_ref().expect("missing anchor field");
    let autocomplete_padding = 5.0;
    let size = request.size.as_ref().expect("missing size field");

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    let mut events = vec![WindowEvent::UpdateWindowGeometry {
        position: None,
        size: Some(LogicalSize::new(size.width.into(), size.height.into())),
        anchor: Some(LogicalSize::new(
            anchor.x.into(),
            (anchor.y + autocomplete_padding).into(),
        )),
        tx: Some(tx),
        dry_run,
    }];

    if !dry_run {
        events.push(
            // Workaround to nonapplicably zero sized windows
            if size.width == 1.0 || size.height == 1.0 {
                WindowEvent::Hide
            } else {
                WindowEvent::Show
            },
        );
    }

    proxy
        .send_event(Event::WindowEvent {
            window_id,
            window_event: WindowEvent::Batch(events),
        })
        .unwrap();

    match rx.recv().await {
        Some((is_above, is_clipped)) => RequestResult::Ok(Box::new(
            ServerOriginatedSubMessage::PositionWindowResponse(PositionWindowResponse {
                is_above: Some(is_above),
                is_clipped: Some(is_clipped),
            }),
        )),
        None => RequestResult::error("unable to determine is_above and is_clipped"),
    }
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
