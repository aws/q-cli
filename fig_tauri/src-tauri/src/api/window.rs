use tauri::{LogicalSize, PhysicalPosition, Position, Size};

use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{PositionWindowRequest, PositionWindowResponse};

use crate::state::{Point, UIState, STATE};
use crate::{api::ResponseKind, os::native};

use super::ResponseResult;

#[allow(unused_variables)]
pub fn update_app_positioning(anchor: Point) {
    let state = native::uiautomation::get_ui_state();

    match state {
        UIState::Focused {
            caret,
            window,
            screen,
        } => {
            let window = (*STATE.window.read().unwrap())
                .clone()
                .expect("Failed to access Tauri window");
            window
                .set_position(Position::Physical(PhysicalPosition {
                    x: caret.x + anchor.x,
                    y: caret.y + anchor.y,
                }))
                .unwrap();
        }
        UIState::Unfocused => {}
    };
}

/// TODO(vikram): implement is_above, is_clipped and corresponding window behavior
pub async fn position_window(request: PositionWindowRequest, _message_id: i64) -> ResponseResult {
    let anchor = request.anchor.expect("Missing anchor field");
    let size = request.size.as_ref().expect("Missing size field");

    let anchor_point = Point {
        x: anchor.x as i32,
        y: anchor.y as i32,
    };

    let dryrun: bool = request.dryrun.unwrap_or(false);

    if !dryrun {
        let window = (*STATE.window.read().unwrap())
            .clone()
            .expect("Failed to access Tauri window");
        window
            .set_size(Size::Logical(LogicalSize {
                width: size.width as f64,
                height: size.height as f64,
            }))
            .unwrap();

        if size.height == 1.0 {
            window.hide().expect("Failed to hide Tauri window");
        } else {
            window.show().expect("Failed to show Tauri window");
        }

        *(STATE.anchor.write().unwrap()) = anchor_point.clone();
        update_app_positioning(anchor_point);
    }

    Ok(ResponseKind::from(
        ServerOriginatedSubMessage::PositionWindowResponse(PositionWindowResponse {
            is_above: Some(false),
            is_clipped: Some(false),
        }),
    ))
}
