use fig_proto::fig::UpdateApplicationPropertiesRequest;
use fig_settings::keybindings::{
    KeyBinding,
    KeyBindings,
};
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

    let request_session_id = request.current_terminal_session_id.map(FigtermSessionId);

    let session_data = figterm_state.with_maybe_id(&request_session_id, |session| {
        (session.sender.clone(), session.id.clone())
    });

    let key_bindings = KeyBindings::load_from_settings("autocomplete")
        .map(|key_bindings| key_bindings.into_iter())
        .unwrap_or_else(|err| {
            error!(%err, "Failed to load keybindings");
            vec![].into_iter()
        })
        .map(|KeyBinding { identifier, binding }| fig_proto::figterm::Action {
            identifier,
            bindings: vec![binding],
        });

    let actions = request
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
        .chain(key_bindings)
        .collect::<Vec<_>>();

    match session_data {
        Some((session_sender, session_id)) => {
            if let Err(err) = session_sender.send(FigtermCommand::InterceptFigJs {
                intercept_keystrokes: request.intercept_bound_keystrokes.unwrap_or_default(),
                intercept_global_keystrokes: request.intercept_global_keystrokes.unwrap_or_default(),
                actions,
            }) {
                error!("Failed sending command to figterm session {session_id}: {err}");
            }
        },
        None => error!(
            ?request_session_id,
            "Failed to send command to figterm session since there is None for Id"
        ),
    }

    RequestResult::success()
}
