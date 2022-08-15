use anyhow::anyhow;
use fig_proto::fig::InsertTextRequest;

use super::{
    RequestResult,
    RequestResultImpl,
};
use crate::figterm::{
    FigTermCommand,
    FigtermSessionId,
    FigtermState,
};

pub async fn insert_text(request: InsertTextRequest, state: &FigtermState) -> RequestResult {
    let figterm_command = match request.r#type {
        Some(some) => match some {
            fig_proto::fig::insert_text_request::Type::Text(text) => FigTermCommand::InsertText {
                insertion: Some(text),
                deletion: None,
                immediate: None,
                offset: None,
                insertion_buffer: None,
            },
            fig_proto::fig::insert_text_request::Type::Update(update) => FigTermCommand::InsertText {
                insertion: update.insertion,
                deletion: update.deletion,
                immediate: update.immediate,
                offset: update.offset,
                insertion_buffer: update.insertion_buffer,
            },
        },
        None => todo!(),
    };

    if let Some(terminal_session_id) = request.terminal_session_id {
        match state.sessions.get(&FigtermSessionId(terminal_session_id)) {
            Some(session) => {
                session
                    .sender
                    .send(figterm_command)
                    .await
                    .map_err(|_| anyhow!("Failed sending command to figterm session"))?;
                return RequestResult::success();
            },
            None => return RequestResult::error("No terminal session with specificed id"),
        }
    }

    if let Some(session) = state.most_recent_session() {
        session
            .sender
            .send(figterm_command)
            .await
            .map_err(|_| anyhow!("Failed sending command to figterm session"))?;
    } else {
        return RequestResult::error("No figterm sessions");
    }

    RequestResult::success()
}
