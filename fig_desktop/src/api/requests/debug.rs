use fig_proto::fig::DebuggerUpdateRequest;
use tracing::info;

use super::{
    RequestResult,
    RequestResultImpl,
};
use crate::DebugState;

pub async fn update(request: DebuggerUpdateRequest, state: &DebugState) -> RequestResult {
    for message in &request.layout {
        if !message.is_empty() {
            info!("{message}");
        } else {
            info!("Message cleared");
        }
    }

    *state.debug_lines.write() = request.layout;
    *state.color.write() = request.color;

    RequestResult::success()
}
