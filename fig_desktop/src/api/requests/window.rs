use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{
    FocusAction,
    PositionWindowRequest,
    PositionWindowResponse,
    WindowFocusRequest,
};

use super::{
    RequestResult,
    RequestResultImpl,
};
use crate::event::{
    Event,
    WindowEvent,
};
use crate::window::WindowId;
use crate::EventLoopProxy;

/// TODO(vikram): implement is_above, is_clipped and corresponding window behavior
pub async fn position_window(
    request: PositionWindowRequest,
    window_id: WindowId,
    proxy: &EventLoopProxy,
) -> RequestResult {
    let dryrun = request.dryrun.unwrap_or(false);

    if !dryrun {
        let anchor = request.anchor.expect("Missing anchor field");
        let size = request.size.as_ref().expect("Missing size field");

        proxy
            .send_event(Event::WindowEvent {
                window_id: window_id.clone(),
                window_event: WindowEvent::Resize {
                    width: size.width as u32,
                    height: size.height as u32,
                },
            })
            .unwrap();

        proxy
            .send_event(Event::WindowEvent {
                window_id: window_id.clone(),
                window_event: WindowEvent::Reanchor {
                    x: anchor.x as i32,
                    y: anchor.y as i32,
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
                        window_event: WindowEvent::HideSoft,
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
    }

    RequestResult::Ok(Box::new(ServerOriginatedSubMessage::PositionWindowResponse(
        PositionWindowResponse {
            is_above: Some(false),
            is_clipped: Some(false),
        },
    )))
}

pub async fn focus(request: WindowFocusRequest, window_id: WindowId, proxy: &EventLoopProxy) -> RequestResult {
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
