use crate::api::FIG_PROTO_MESSAGE_RECIEVED;
use crate::state::figterm::FigtermSessionId;
use crate::{api::window::update_app_positioning, local::figterm::ensure_figterm, state::STATE};
use anyhow::Result;
use bytes::BytesMut;
use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::KeyEvent;
use fig_proto::local::{FocusChangeHook, PreExecHook};
use fig_proto::{
    fig::{
        EditBufferChangedNotification, KeybindingPressedNotification, Notification,
        NotificationType, ServerOriginatedMessage,
    },
    local::{CursorPositionHook, EditBufferHook, InterceptedKeyHook, PromptHook},
    prost::Message,
};
use tracing::debug;

pub async fn send_notification(
    notification_type: &NotificationType,
    notification: Notification,
) -> Result<()> {
    let message_id = match STATE.subscriptions.get(notification_type) {
        Some(id) => *id,
        None => {
            return Ok(());
        }
    };

    let message = ServerOriginatedMessage {
        id: Some(message_id),
        submessage: Some(ServerOriginatedSubMessage::Notification(notification)),
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

pub async fn edit_buffer(hook: EditBufferHook) -> Result<()> {
    let session_id = FigtermSessionId(hook.context.clone().unwrap().session_id.unwrap());
    ensure_figterm(session_id.clone());

    STATE.figterm_state.with_mut(session_id.clone(), |session| {
        session.edit_buffer.text = hook.text.clone();
        session.edit_buffer.cursor = hook.cursor;
        session.context = hook.context.clone();
    });

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
                    session_id: Some(session_id.0),
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

pub async fn pre_exec(_: PreExecHook) -> Result<()> {
    Ok(())
}

pub async fn intercepted_key(intercepted_key_hook: InterceptedKeyHook) -> Result<()> {
    debug!(
        "intercepted_key: {:?}, {:x?}",
        intercepted_key_hook, intercepted_key_hook.key
    );

    let action = match intercepted_key_hook.key.as_str() {
        "\u{1b}OA" => "navigateUp",
        "\u{1b}OB" => "navigateDown",
        "\r" => "insertSelected",
        "\t" => "insertCommonPrefix",
        _ => "",
    };

    send_notification(
        &NotificationType::NotifyOnKeybindingPressed,
        Notification {
            r#type: Some(
                fig_proto::fig::notification::Type::KeybindingPressedNotification(
                    KeybindingPressedNotification {
                        keypress: Some(KeyEvent {
                            characters: Some(intercepted_key_hook.key),
                            ..Default::default()
                        }),
                        action: Some(action.to_string()),
                    },
                ),
            ),
        },
    )
    .await
    .unwrap();
    Ok(())
}
