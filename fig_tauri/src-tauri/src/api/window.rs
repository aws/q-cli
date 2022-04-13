use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{PositionWindowRequest, PositionWindowResponse};

use crate::api::ResponseKind;

use super::ResponseResult;

pub async fn position_window(_request: PositionWindowRequest, _message_id: i64) -> ResponseResult {
    // TODO: Full implementation
    Ok(ResponseKind::from(
        ServerOriginatedSubMessage::PositionWindowResponse(PositionWindowResponse {
            is_above: Some(false),
            is_clipped: Some(false),
        }),
    ))
}
