use crate::api::FIG_PROTO_MESSAGE_RECIEVED;
use crate::{api::window::update_app_positioning, local::figterm::ensure_figterm, state::STATE};
use anyhow::Result;
use bytes::BytesMut;
use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::local::FocusChangeHook;
use fig_proto::{
    fig::{EditBufferChangedNotification, Notification, NotificationType, ServerOriginatedMessage},
    local::{CursorPositionHook, EditBufferHook, PromptHook},
    prost::Message,
};
use tracing::info;

pub async fn edit_buffer(hook: EditBufferHook) -> Result<()> {
    info!("WE'RE GETTING EDIT BUFFERS");
    let session_id = hook.context.clone().unwrap().session_id.unwrap();
    ensure_figterm(session_id.clone());
    let mut session = STATE.figterm_sessions.get_mut(&session_id).unwrap();
    session.edit_buffer.text = hook.text.clone();
    session.edit_buffer.cursor = hook.cursor;

    let message_id = match STATE
        .subscriptions
        .get(&NotificationType::NotifyOnEditbuffferChange)
    {
        Some(id) => *id,
        None => {
            return Ok(());
        }
    };

    let message = ServerOriginatedMessage {
        id: Some(message_id),
        submessage: Some(ServerOriginatedSubMessage::Notification(Notification {
            r#type: Some(fig_proto::fig::notification::Type::EditBufferNotification(
                EditBufferChangedNotification {
                    context: hook.context,
                    buffer: Some(hook.text),
                    cursor: Some(hook.cursor.try_into().unwrap()),
                    session_id: Some(session_id),
                },
            )),
        })),
    };

    let mut encoded = BytesMut::new();
    message.encode(&mut encoded).unwrap();

    let window = (*STATE.window.read().unwrap())
        .clone()
        .expect("Failed to access Tauri window");
    window
        .emit(FIG_PROTO_MESSAGE_RECIEVED, base64::encode(encoded))
        .expect("Failed to emit edit buffer notification");

    update_app_positioning((*STATE.anchor.read().unwrap()).clone());

    Ok(())
}

pub async fn cursor_position(hook: CursorPositionHook) -> Result<()> {
    let mut handle = STATE.cursor_position.lock();
    handle.x = hook.x;
    handle.y = hook.y;
    handle.width = hook.width;
    handle.height = hook.height;
    Ok(())
}

pub async fn prompt(_: PromptHook) -> Result<()> {
    Ok(())
}

pub async fn focus_change(_: FocusChangeHook) -> Result<()> {
    Ok(())
}
