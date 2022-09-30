use fig_proto::fig::InsertTextRequest;

use super::{
    RequestResult,
    RequestResultImpl,
};
use crate::figterm::{
    FigtermCommand,
    FigtermSessionId,
    FigtermState,
};

pub async fn insert_text(request: InsertTextRequest, state: &FigtermState) -> RequestResult {
    let figterm_command = match request.r#type {
        Some(some) => match some {
            fig_proto::fig::insert_text_request::Type::Text(text) => FigtermCommand::InsertText {
                insertion: Some(text),
                deletion: None,
                immediate: None,
                offset: None,
                insertion_buffer: None,
            },
            fig_proto::fig::insert_text_request::Type::Update(update) => FigtermCommand::InsertText {
                insertion: update.insertion,
                deletion: update.deletion,
                immediate: update.immediate,
                offset: update.offset,
                insertion_buffer: update.insertion_buffer,
            },
        },
        None => return RequestResult::error("InsertTextRequest expects a request type"),
    };

    match state.with_maybe_id(&request.terminal_session_id.map(FigtermSessionId), |session| {
        session.sender.clone()
    }) {
        Some(sender) => {
            sender
                .send(figterm_command)
                .map_err(|_| "Failed sending command to figterm session")?;
            RequestResult::success()
        },
        None => RequestResult::error("No figterm sessions"),
    }
}
