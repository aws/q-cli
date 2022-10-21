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
    EditBufferHook,
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

use crate::event::{
    EmitEventName,
    Event,
    WindowEvent,
};
use crate::figterm::{
    FigtermSessionId,
    FigtermState,
    SessionMetrics,
};
use crate::platform::PlatformBoundEvent;
use crate::webview::notification::WebviewNotificationsState;
use crate::{
    EventLoopProxy,
    AUTOCOMPLETE_ID,
};

pub async fn edit_buffer(
    hook: &EditBufferHook,
    session_id: &FigtermSessionId,
    figterm_state: Arc<FigtermState>,
    notifications_state: &WebviewNotificationsState,
    proxy: &EventLoopProxy,
) -> Result<()> {
    let old_metrics = figterm_state.with_update(session_id.clone(), |session| {
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
                    (metrics.end_time - metrics.start_time).whole_seconds().into(),
                ),
                ("num_insertions", metrics.num_insertions.into()),
                ("num_popups", metrics.num_popups.into()),
            ];
            tokio::spawn(async {
                if let Err(err) = fig_telemetry::emit_track(fig_telemetry::TrackEvent::new(
                    fig_telemetry::TrackEventType::TerminalSessionMetricsRecorded,
                    fig_telemetry::TrackSource::Desktop,
                    env!("CARGO_PKG_VERSION").into(),
                    properties,
                ))
                .await
                {
                    warn!(%err, "Failed to record terminal session metrics");
                }
            });
        }
    }

    let utf16_cursor_position = hook
        .text
        .get(..hook.cursor as usize)
        .map(|s| s.encode_utf16().count() as i32);

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
                        cursor: utf16_cursor_position,
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
                    event_name: EmitEventName::Notification,
                    payload: base64::encode(encoded),
                },
            })
            .unwrap();
    }

    proxy.send_event(Event::PlatformBoundEvent(PlatformBoundEvent::EditBufferChanged))?;

    proxy.send_event(Event::WindowEvent {
        window_id: AUTOCOMPLETE_ID,
        // If editbuffer is empty, hide the autocomplete window to avoid flickering
        window_event: if hook.text.is_empty() {
            WindowEvent::Hide
        } else {
            WindowEvent::Show
        },
    })?;

    Ok(())
}

pub async fn prompt(
    hook: &PromptHook,
    session_id: &FigtermSessionId,
    figterm_state: &FigtermState,
    notifications_state: &WebviewNotificationsState,
    proxy: &EventLoopProxy,
) -> Result<()> {
    figterm_state.with(session_id, |session| {
        session.context = hook.context.clone();
    });

    notifications_state
        .broadcast_notification_all(
            &NotificationType::NotifyOnPrompt,
            Notification {
                r#type: Some(fig_proto::fig::notification::Type::ShellPromptReturnedNotification(
                    ShellPromptReturnedNotification {
                        session_id: Some(session_id.0.clone()),
                        shell: hook.context.as_ref().map(|ctx| Process {
                            pid: ctx.pid,
                            executable: ctx.process_name.clone(),
                            directory: ctx.current_working_directory.clone(),
                            env: vec![],
                        }),
                    },
                )),
            },
            proxy,
        )
        .await
}

pub async fn pre_exec(
    hook: &PreExecHook,
    session_id: &FigtermSessionId,
    figterm_state: &FigtermState,
    notifications_state: &WebviewNotificationsState,
    proxy: &EventLoopProxy,
) -> Result<()> {
    figterm_state.with_update(session_id.clone(), |session| {
        session.context = hook.context.clone();
    });

    proxy.send_event(Event::WindowEvent {
        window_id: AUTOCOMPLETE_ID.clone(),
        window_event: WindowEvent::Hide,
    })?;

    notifications_state
        .broadcast_notification_all(
            &NotificationType::NotifyOnProcessChanged,
            Notification {
                r#type: Some(fig_proto::fig::notification::Type::ProcessChangeNotification(
                    ProcessChangedNotification {
                    session_id: Some(session_id.0.clone()),
                    new_process: // TODO: determine active application based on tty
                    hook.context.as_ref().map(|ctx| Process {
                        pid: ctx.pid,
                        executable: ctx.process_name.clone(),
                        directory: ctx.current_working_directory.clone(),
                        env: vec![],
                    }),
                },
                )),
            },
            proxy,
        )
        .await
}

pub async fn intercepted_key(
    InterceptedKeyHook { action, context, .. }: InterceptedKeyHook,
    notifications_state: &WebviewNotificationsState,
    proxy: &EventLoopProxy,
) -> Result<()> {
    debug!(%action, "Intercepted Key Action");

    notifications_state
        .broadcast_notification_all(
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
}
