use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{
    PositionWindowRequest,
    PositionWindowResponse,
};

use super::RequestResult;
use crate::window::{
    WindowEvent,
    WindowState,
};

/// TODO(vikram): implement is_above, is_clipped and corresponding window behavior
pub async fn position_window(request: PositionWindowRequest, state: &WindowState) -> RequestResult {
    let dryrun: bool = request.dryrun.unwrap_or(false);

    if !dryrun {
        let anchor = request.anchor.expect("Missing anchor field");
        let size = request.size.as_ref().expect("Missing size field");

        state.send_event(WindowEvent::Reanchor {
            x: anchor.x as i32,
            y: anchor.y as i32,
        });

        state.send_event(WindowEvent::Resize {
            width: size.width as u32,
            height: size.height as u32,
        });

        // Workaround to nonapplicably zero sized windows
        match size.width == 1.0 || size.height == 1.0 {
            true => state.send_event(WindowEvent::Hide),
            false => state.send_event(WindowEvent::Show),
        }
    }

    RequestResult::Ok(Box::new(ServerOriginatedSubMessage::PositionWindowResponse(
        PositionWindowResponse {
            is_above: Some(false),
            is_clipped: Some(false),
        },
    )))
}
