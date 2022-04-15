use fig_proto::fig::DebuggerUpdateRequest;
use tracing::info;

use super::{ResponseKind, ResponseResult};

pub async fn update(request: DebuggerUpdateRequest, _message_id: i64) -> ResponseResult {
    for message in request.layout {
        if message.len() > 0 {
            info!("{}", message);
        }
    }

    Ok(ResponseKind::Success)
}
