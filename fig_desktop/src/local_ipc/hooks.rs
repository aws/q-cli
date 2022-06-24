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
    Process,
    ProcessChangedNotification,
    ServerOriginatedMessage,
    ShellPromptReturnedNotification,
};
use fig_proto::local::{
    CursorPositionHook,
    EditBufferHook,
    FileChangedHook,
    FocusChangeHook,
    InterceptedKeyHook,
    PreExecHook,
    PromptHook,
};
use fig_proto::prost::Message;
use tracing::debug;

use crate::event::{
    NativeEvent,
    WindowEvent,
};
use crate::figterm::{
    ensure_figterm,
    FigtermSessionId,
};
use crate::{
    Event,
    EventLoopProxy,
    GlobalState,
    NotificationsState,
    AUTOCOMPLETE_ID,
    FIG_PROTO_MESSAGE_RECIEVED,
};

pub async fn send_notification(
    notification_type: &NotificationType,
    notification: Notification,
    notification_state: &NotificationsState,
    proxy: &EventLoopProxy,
) -> Result<()> {
    for sub in notification_state.subscriptions.iter() {
        let message_id = match sub.get(notification_type) {
            Some(id) => *id,
            None => continue,
        };

        let message = ServerOriginatedMessage {
            id: Some(message_id),
            submessage: Some(ServerOriginatedSubMessage::Notification(notification.clone())),
        };

        let mut encoded = BytesMut::new();
        message.encode(&mut encoded).unwrap();

        proxy
            .send_event(Event::WindowEvent {
                window_id: sub.key().clone(),
                window_event: WindowEvent::Emit {
                    event: FIG_PROTO_MESSAGE_RECIEVED.into(),
                    payload: base64::encode(encoded),
                },
            })
            .unwrap()
    }

    Ok(())
}

pub async fn edit_buffer(hook: EditBufferHook, global_state: Arc<GlobalState>, proxy: &EventLoopProxy) -> Result<()> {
    let session_id = FigtermSessionId(hook.context.clone().unwrap().session_id.unwrap());
    ensure_figterm(session_id.clone(), global_state.clone());

    global_state.figterm_state.with_mut(session_id.clone(), |session| {
        session.edit_buffer.text = hook.text.clone();
        session.edit_buffer.cursor = hook.cursor;
        session.context = hook.context.clone();
    });

    for sub in global_state.notifications_state.subscriptions.iter() {
        let message_id = match sub.get(&NotificationType::NotifyOnEditbuffferChange) {
            Some(id) => *id,
            None => continue,
        };

        let hook = hook.clone();
        let session_id = session_id.clone();
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

        proxy
            .send_event(Event::WindowEvent {
                window_id: sub.key().clone(),
                window_event: WindowEvent::Emit {
                    event: FIG_PROTO_MESSAGE_RECIEVED.into(),
                    payload: base64::encode(encoded),
                },
            })
            .unwrap();
    }

    proxy.send_event(Event::NativeEvent(NativeEvent::EditBufferChanged))?;

    proxy.send_event(Event::WindowEvent {
        window_id: AUTOCOMPLETE_ID,
        window_event: WindowEvent::Show,
    })?;

    Ok(())
}

pub async fn caret_position(
    CursorPositionHook { x, y, width, height }: CursorPositionHook,
    proxy: &EventLoopProxy,
) -> Result<()> {
    debug!("Cursor Position: {x} {y} {width} {height}");

    proxy
        .send_event(Event::WindowEvent {
            window_id: AUTOCOMPLETE_ID.clone(),
            window_event: WindowEvent::UpdateCaret { x, y, width, height },
        })
        .unwrap();

    Ok(())
}

pub async fn prompt(hook: PromptHook, global_state: &GlobalState, proxy: &EventLoopProxy) -> Result<()> {
    send_notification(
        &NotificationType::NotifyOnPrompt,
        Notification {
            r#type: Some(fig_proto::fig::notification::Type::ShellPromptReturnedNotification(
                ShellPromptReturnedNotification {
                    session_id: hook.context.as_ref().and_then(|ctx| ctx.session_id.clone()),
                    shell: hook.context.map(|ctx| Process {
                        pid: ctx.pid,
                        executable: ctx.process_name,
                        directory: ctx.current_working_directory,
                        env: vec![],
                    }),
                },
            )),
        },
        &global_state.notifications_state,
        proxy,
    )
    .await
    .unwrap();
    Ok(())
}

pub async fn pre_exec(hook: PreExecHook, global_state: &GlobalState, proxy: &EventLoopProxy) -> Result<()> {
    send_notification(
        &NotificationType::NotifyOnProcessChanged,
        Notification {
            r#type: Some(fig_proto::fig::notification::Type::ProcessChangeNotification(
                ProcessChangedNotification {
                    session_id: hook.context.as_ref().and_then(|ctx| ctx.session_id.clone()),
                    new_process: // TODO: determine active application based on tty
                    hook.context.map(|ctx| Process {
                        pid: ctx.pid,
                        executable: ctx.process_name,
                        directory: ctx.current_working_directory,
                        env: vec![],
                    }),
                },
            )),
        },
        &global_state.notifications_state,
        proxy,
    )
    .await
    .unwrap();
    Ok(())
}

pub async fn focus_change(_: FocusChangeHook, proxy: &EventLoopProxy) -> Result<()> {
    proxy
        .send_event(Event::WindowEvent {
            window_id: AUTOCOMPLETE_ID.clone(),
            window_event: WindowEvent::Hide,
        })
        .unwrap();

    Ok(())
}

pub async fn intercepted_key(
    InterceptedKeyHook { key, action, .. }: InterceptedKeyHook,
    global_state: &GlobalState,
    proxy: &EventLoopProxy,
) -> Result<()> {
    debug!("Intercepted Key Action: {:?}", action);

    send_notification(
        &NotificationType::NotifyOnKeybindingPressed,
        Notification {
            r#type: Some(fig_proto::fig::notification::Type::KeybindingPressedNotification(
                KeybindingPressedNotification {
                    keypress: Some(KeyEvent {
                        characters: Some(key),
                        ..Default::default()
                    }),
                    action: Some(action),
                },
            )),
        },
        &global_state.notifications_state,
        proxy,
    )
    .await
    .unwrap();

    Ok(())
}

pub async fn file_changed(_file_changed_hook: FileChangedHook) -> Result<()> {
    Ok(())
}
