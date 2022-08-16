use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use bytes::BytesMut;
use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{
    EditBufferChangedNotification,
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
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use tracing::{
    debug,
    warn,
};

use crate::event::WindowEvent;
use crate::figterm::{
    ensure_figterm,
    FigtermSessionId,
    FigtermState,
    SessionMetrics,
};
use crate::notification::NotificationsState;
use crate::{
    Event,
    EventLoopProxy,
    AUTOCOMPLETE_ID,
    FIG_PROTO_MESSAGE_RECIEVED,
};

pub async fn edit_buffer(
    hook: EditBufferHook,
    figterm_state: Arc<FigtermState>,
    notifications_state: &NotificationsState,
    proxy: &EventLoopProxy,
) -> Result<()> {
    let session_id = FigtermSessionId(hook.context.clone().unwrap().session_id.unwrap());
    ensure_figterm(session_id.clone(), figterm_state.clone())?;

    let old_metrics = figterm_state.with_mut(session_id.clone(), |session| {
        session.edit_buffer.text = hook.text.clone();
        session.edit_buffer.cursor = hook.cursor;
        session.terminal_cursor_coordinates = hook.terminal_cursor_coordinates.clone();
        session.context = hook.context.clone();

        let received_at = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
        let current_session_expired = session
            .current_session_metrics
            .as_ref()
            .map(|metrics| received_at > metrics.end_time + Duration::from_secs(5))
            .unwrap_or(true);

        if current_session_expired {
            let previous = session.current_session_metrics.clone();
            session.current_session_metrics = Some(SessionMetrics::new(received_at));
            previous
        } else {
            if let Some(ref mut metrics) = session.current_session_metrics {
                metrics.end_time = received_at;
            }
            None
        }
    });

    if let Some(metrics) = old_metrics.flatten() {
        if metrics.end_time > metrics.start_time {
            let properties: Vec<(&str, serde_json::Value)> = vec![
                ("start_time", metrics.start_time.format(&Rfc3339)?.into()),
                ("end_time", metrics.end_time.format(&Rfc3339)?.into()),
                (
                    "duration",
                    (metrics.start_time - metrics.end_time).whole_seconds().into(),
                ),
                ("num_insertions", metrics.num_insertions.into()),
                ("num_popups", metrics.num_popups.into()),
            ];
            tokio::spawn(async {
                if let Err(e) = fig_telemetry::emit_track(fig_telemetry::TrackEvent::new(
                    fig_telemetry::TrackEventType::TerminalSessionMetricsRecorded,
                    fig_telemetry::TrackSource::App,
                    properties,
                ))
                .await
                {
                    warn!("Failed to record terminal session metrics: {e}");
                }
            });
        }
    }

    for sub in notifications_state.subscriptions.iter() {
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

    #[cfg(target_os = "windows")]
    proxy.send_event(Event::NativeEvent(crate::event::NativeEvent::EditBufferChanged))?;

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
            window_event: WindowEvent::Reposition { x, y: y + height },
        })
        .unwrap();

    Ok(())
}

pub async fn prompt(hook: PromptHook, notifications_state: &NotificationsState, proxy: &EventLoopProxy) -> Result<()> {
    notifications_state
        .send_notification(
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
            proxy,
        )
        .await
        .unwrap();
    Ok(())
}

pub async fn pre_exec(
    hook: PreExecHook,
    notifications_state: &NotificationsState,
    proxy: &EventLoopProxy,
) -> Result<()> {
    notifications_state
        .send_notification(
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
    InterceptedKeyHook { action, context, .. }: InterceptedKeyHook,
    notifications_state: &NotificationsState,
    proxy: &EventLoopProxy,
) -> Result<()> {
    debug!(?action, "Intercepted Key Action");

    notifications_state
        .send_notification(
            &NotificationType::NotifyOnKeybindingPressed,
            Notification {
                r#type: Some(fig_proto::fig::notification::Type::KeybindingPressedNotification(
                    KeybindingPressedNotification {
                        keypress: None,
                        action: Some(action),
                        context,
                    },
                )),
            },
            proxy,
        )
        .await
        .unwrap();

    Ok(())
}

pub async fn file_changed(_file_changed_hook: FileChangedHook) -> Result<()> {
    Ok(())
}
