use fig_proto::fig::DebuggerUpdateRequest;
use tracing::info;

use crate::state::STATE;

use super::{ResponseKind, ResponseResult};

pub async fn update(request: DebuggerUpdateRequest, _message_id: i64) -> ResponseResult {
    for message in &request.layout {
        if !message.is_empty() {
            info!("{}", message);
        }
    }
    *STATE.debug_state.debug_lines.write() = request.layout;
    *STATE.debug_state.color.write() = request.color;
    Ok(ResponseKind::Success)
}
