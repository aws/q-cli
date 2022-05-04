use fig_proto::fig::DebuggerUpdateRequest;
use tracing::info;

use crate::DebugState;

use super::RequestResult;
use super::RequestResultImpl;

pub async fn update(request: DebuggerUpdateRequest, state: &DebugState) -> RequestResult {
    for message in &request.layout {
        if !message.is_empty() {
            info!("{}", message);
        }
    }

    *state.debug_lines.write() = request.layout;
    *state.color.write() = request.color;

    RequestResult::success()
}
