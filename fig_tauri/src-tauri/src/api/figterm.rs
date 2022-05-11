use anyhow::anyhow;
use fig_proto::fig::InsertTextRequest;

use super::{
    RequestResult,
    RequestResultImpl,
};
use crate::figterm::{
    FigTermCommand,
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
            },
            fig_proto::fig::insert_text_request::Type::Update(update) => FigTermCommand::InsertText {
                insertion: update.insertion,
                deletion: update.deletion,
                immediate: update.immediate,
                offset: update.offset,
            },
        },
        None => todo!(),
    };

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
