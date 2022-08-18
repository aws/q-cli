use fig_proto::fig::UpdateApplicationPropertiesRequest;
use tracing::error;

use super::{
    RequestResult,
    RequestResultImpl,
};
use crate::figterm::{
    FigTermCommand,
    FigtermSessionId,
    FigtermState,
};
use crate::InterceptState;

pub async fn update(
    request: UpdateApplicationPropertiesRequest,
    figterm_state: &FigtermState,
    intercept_state: &InterceptState,
) -> RequestResult {
    if let Some(intercept_bound_keystrokes) = request.intercept_bound_keystrokes {
        *intercept_state.intercept_bound_keystrokes.write() = intercept_bound_keystrokes;
    }

    if let Some(intercept_global_keystrokes) = request.intercept_global_keystrokes {
        *intercept_state.intercept_global_keystrokes.write() = intercept_global_keystrokes;
    }

    let lock_session_id = request
        .current_terminal_session_id
        .map(FigtermSessionId)
        .or_else(|| figterm_state.most_recent_session_id());

    for session in figterm_state.sessions.iter() {
        match lock_session_id {
            Some(ref lock_session_id) if session.key() == lock_session_id => {
                if let Some(session) = figterm_state.sessions.get(lock_session_id) {
                    if let Err(err) = session.sender.send(FigTermCommand::InterceptFigJs {
                        intercept_bound_keystrokes: request.intercept_bound_keystrokes.unwrap_or_default(),
                        intercept_global_keystrokes: request.intercept_bound_keystrokes.unwrap_or_default(),
                        actions: request
                            .action_list
                            .clone()
                            .into_iter()
                            .flat_map(|list| {
                                list.actions.into_iter().filter_map(|action| {
                                    action.identifier.map(|identifier| fig_proto::figterm::Action {
                                        identifier,
                                        bindings: action.default_bindings,
                                    })
                                })
                            })
                            .collect(),
                    }) {
                        error!("Failed sending command to figterm session {lock_session_id}: {err}");
                    }
                }
            },
            _ => {
                if let Err(err) = session.sender.send(FigTermCommand::InterceptClear) {
                    let key = session.key();
                    error!("Failed sending command to figterm session {key}: {err}");
                }
            },
        }
    }

    RequestResult::success()
}
