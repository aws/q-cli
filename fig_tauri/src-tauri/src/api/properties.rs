use fig_proto::fig::UpdateApplicationPropertiesRequest;
use tracing::{error, trace};

use crate::{local::figterm::FigTermCommand, state::STATE};

use super::{ResponseKind, ResponseResult};

pub async fn update(
    request: UpdateApplicationPropertiesRequest,
    _message_id: i64,
) -> ResponseResult {
    if let Some(intercept_bound_keystrokes) = request.intercept_bound_keystrokes {
        *STATE.key_intercept_state.intercept_bound_keystrokes.write() = intercept_bound_keystrokes;
        trace!("intercept_bound_keystrokes: {}", intercept_bound_keystrokes);
    }

    if let Some(intercept_global_keystrokes) = request.intercept_global_keystrokes {
        *STATE
            .key_intercept_state
            .intercept_global_keystrokes
            .write() = intercept_global_keystrokes;
        trace!(
            "intercept_global_keystrokes: {}",
            intercept_global_keystrokes
        );

        if intercept_global_keystrokes {
            if let Some(session) = STATE.figterm_state.most_recent_session() {
                if let Err(err) = session.sender.send(FigTermCommand::SetInterceptAll).await {
                    error!("Failed sending command to figterm session: {}", err);
                }
            }
        } else {
            for session in STATE.figterm_state.sessions.iter() {
                if let Err(err) = session.sender.send(FigTermCommand::ClearIntercept).await {
                    error!(
                        "Failed sending command to figterm session {}: {}",
                        session.key(),
                        err
                    );
                }
            }
        }
    }

    // TODO: Handle actionList

    Ok(ResponseKind::Success)
}
