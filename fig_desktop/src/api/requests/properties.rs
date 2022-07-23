use fig_proto::fig::UpdateApplicationPropertiesRequest;
use tracing::error;

use super::{
    RequestResult,
    RequestResultImpl,
};
use crate::figterm::{
    FigTermCommand,
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

    let most_recent_session_id = figterm_state.most_recent_session_id();
    for session in figterm_state.sessions.iter() {
        match most_recent_session_id {
            Some(ref most_recent_session_id) if session.key() == most_recent_session_id => {
                if let Some(session) = figterm_state.sessions.get(most_recent_session_id) {
                    if let Err(err) = session
                        .sender
                        .send(FigTermCommand::InterceptFigJs {
                            intercept_bound_keystrokes: request.intercept_bound_keystrokes(),
                            intercept_global_keystrokes: request.intercept_bound_keystrokes(),
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
                        })
                        .await
                    {
                        error!("Failed sending command to figterm session {most_recent_session_id}: {err}");
                    }
                }
            },
            _ => {
                if let Err(err) = session.sender.send(FigTermCommand::InterceptClear).await {
                    let key = session.key();
                    error!("Failed sending command to figterm session {key}: {err}");
                }
            },
        }
    }

    RequestResult::success()
}
