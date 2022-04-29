use fig_proto::fig::InsertTextRequest;

use crate::{local::figterm::FigTermCommand, response_error, state::STATE};

use super::{ResponseKind, ResponseResult};

pub async fn insert_text(request: InsertTextRequest, _message_id: i64) -> ResponseResult {
    let figterm_command = match request.r#type {
        Some(some) => match some {
            fig_proto::fig::insert_text_request::Type::Text(text) => FigTermCommand::InsertText {
                insertion: Some(text),
                deletion: None,
                immediate: None,
                offset: None,
            },
            fig_proto::fig::insert_text_request::Type::Update(update) => {
                FigTermCommand::InsertText {
                    insertion: update.insertion,
                    deletion: update.deletion,
                    immediate: update.immediate,
                    offset: update.offset,
                }
            }
        },
        None => todo!(),
    };

    if let Some(session) = STATE.figterm_state.most_recent_session() {
        session
            .sender
            .send(figterm_command)
            .await
            .map_err(response_error!("Failed sending command to figterm session"))?;
    } else {
        return Err(ResponseKind::Error("No figterm sessions".into()));
    }
    Ok(ResponseKind::Success)
}
