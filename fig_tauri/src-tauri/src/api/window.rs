use tauri::{LogicalSize, PhysicalPosition, Position, Size};

use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{PositionWindowRequest, PositionWindowResponse};

use crate::api::ResponseKind;
use crate::state::{Point, UIState, STATE};

use super::ResponseResult;

pub fn handle_ui_state(new_state: UIState) {
    let mut ui_state_handle = STATE.ui_state.lock();
    let anchor = STATE.anchor.lock().clone();

    if *ui_state_handle != new_state {
        update_app_positioning(new_state.clone(), anchor);
        *ui_state_handle = new_state;
    }
}

fn update_app_positioning(state: UIState, anchor: Point) {
    match state {
        UIState::Focused {
            caret,
            window,
            screen,
        } => {
            STATE
                .window
                .lock()
                .as_ref()
                .unwrap()
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
    let anchor = request.anchor.unwrap();
    let size = request.size.as_ref().unwrap();

    let anchor_point = Point {
        x: anchor.x as i32,
        y: anchor.y as i32,
    };

    let dryrun: bool = request.dryrun.unwrap_or(false);

    if !dryrun {
        let window = STATE.window.lock().clone().unwrap();
        window
            .set_size(Size::Logical(LogicalSize {
                width: size.width as f64,
                height: size.height as f64,
            }))
            .unwrap();

        if size.height == 1.0 {
            window.hide().unwrap();
        } else {
            window.show().unwrap();
        }

        *(STATE.anchor.lock()) = anchor_point.clone();
        update_app_positioning(STATE.ui_state.lock().clone(), anchor_point);
    }

    Ok(ResponseKind::from(
        ServerOriginatedSubMessage::PositionWindowResponse(PositionWindowResponse {
            is_above: Some(false),
            is_clipped: Some(false),
        }),
    ))
}
