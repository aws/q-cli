use fig_proto::fig::DebuggerUpdateRequest;
use tracing::info;

use super::{ResponseKind, ResponseResult};

pub async fn update(request: DebuggerUpdateRequest, _message_id: i64) -> ResponseResult {
    for message in request.layout {
        if !message.is_empty() {
            info!("{}", message);
        }
    }
    Ok(ResponseKind::Success)
}
