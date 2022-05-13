use std::sync::Arc;

use anyhow::Result;
use bytes::BytesMut;
use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{
    EditBufferChangedNotification,
    KeyEvent,
    KeybindingPressedNotification,
    Notification,
    NotificationType,
    ServerOriginatedMessage,
};
use fig_proto::local::{
    CursorPositionHook,
    EditBufferHook,
    FocusChangeHook,
    InterceptedKeyHook,
    PreExecHook,
    PromptHook,
};
use fig_proto::prost::Message;
use tracing::debug;

use crate::figterm::{
    ensure_figterm,
    FigtermSessionId,
    FigtermState,
};
use crate::window::{
    WindowEvent,
    WindowState,
};
use crate::{
    NotificationsState,
    FIG_PROTO_MESSAGE_RECIEVED,
};

pub async fn send_notification(
    notification_type: &NotificationType,
    notification: Notification,
    notification_state: &NotificationsState,
    window_state: &WindowState,
) -> Result<()> {
    let message_id = match notification_state.subscriptions.get(notification_type) {
        Some(id) => *id,
        None => {
            return Ok(());
        },
    };

    let message = ServerOriginatedMessage {
        id: Some(message_id),
        submessage: Some(ServerOriginatedSubMessage::Notification(notification)),
    };

    let mut encoded = BytesMut::new();
    message.encode(&mut encoded).unwrap();

    window_state.send_event(WindowEvent::Emit {
        event: FIG_PROTO_MESSAGE_RECIEVED,
        payload: base64::encode(encoded),
    });

    Ok(())
}

pub async fn edit_buffer(
    hook: EditBufferHook,
    figterm_state: Arc<FigtermState>,
    notification_state: &NotificationsState,
    window_state: &WindowState,
) -> Result<()> {
    let session_id = FigtermSessionId(hook.context.clone().unwrap().session_id.unwrap());
    ensure_figterm(session_id.clone(), figterm_state.clone());

    figterm_state.with_mut(session_id.clone(), |session| {
        session.edit_buffer.text = hook.text.clone();
        session.edit_buffer.cursor = hook.cursor;
        session.context = hook.context.clone();
    });

    let message_id = match notification_state
        .subscriptions
        .get(&NotificationType::NotifyOnEditbuffferChange)
    {
        Some(id) => *id,
        None => {
            return Ok(());
        },
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

    window_state.send_event(WindowEvent::Emit {
        event: FIG_PROTO_MESSAGE_RECIEVED,
        payload: base64::encode(encoded),
    });

    Ok(())
}

pub async fn caret_position(hook: CursorPositionHook, state: &WindowState) -> Result<()> {
    state.send_event(WindowEvent::UpdateCaret {
        x: hook.x,
        y: hook.y,
        width: hook.width,
        height: hook.height,
    });

    Ok(())
}

pub async fn prompt(hook: PromptHook) -> Result<()> {
    Ok(())
}

pub async fn focus_change(hook: FocusChangeHook) -> Result<()> {
    Ok(())
}

pub async fn pre_exec(hook: PreExecHook) -> Result<()> {
    Ok(())
}

pub async fn intercepted_key(
    intercepted_key_hook: InterceptedKeyHook,
    notification_state: &NotificationsState,
    window_state: &WindowState,
) -> Result<()> {
    debug!("Intercepted Key Action: {:?}", intercepted_key_hook.action);

    send_notification(
        &NotificationType::NotifyOnKeybindingPressed,
        Notification {
            r#type: Some(fig_proto::fig::notification::Type::KeybindingPressedNotification(
                KeybindingPressedNotification {
                    keypress: Some(KeyEvent {
                        characters: Some(intercepted_key_hook.key),
                        ..Default::default()
                    }),
                    action: Some(intercepted_key_hook.action),
                },
            )),
        },
        notification_state,
        window_state,
    )
    .await
    .unwrap();

    Ok(())
}
