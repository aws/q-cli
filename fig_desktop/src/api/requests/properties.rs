use fig_proto::fig::UpdateApplicationPropertiesRequest;
use tracing::error;

use super::{
    RequestResult,
    RequestResultImpl,
};
use crate::figterm::{
    FigtermCommand,
    FigtermSessionId,
    FigtermState,
};
use crate::InterceptState;

pub fn update(
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

    let session_data = figterm_state
        .with_maybe_id(&request.current_terminal_session_id.map(FigtermSessionId), |session| {
            (session.sender.clone(), session.id.clone())
        });

    if let Some((session_sender, session_id)) = session_data {
        if let Err(err) = session_sender.send(FigtermCommand::InterceptFigJs {
            intercept_bound_keystrokes: request.intercept_bound_keystrokes.unwrap_or_default(),
            intercept_global_keystrokes: request.intercept_bound_keystrokes.unwrap_or_default(),
            actions: request
                .action_list
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
            error!("Failed sending command to figterm session {session_id}: {err}");
        }
    }

    RequestResult::success()
}
