use std::sync::Arc;
use std::time::{
    Duration,
    SystemTime,
};

use anyhow::{
    anyhow,
    Result,
};
use base64::prelude::*;
use bytes::BytesMut;
use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{
    EditBufferChangedNotification,
    HistoryUpdatedNotification,
    KeybindingPressedNotification,
    LocationChangedNotification,
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
    PostExecHook,
    PreExecHook,
    PromptHook,
};
use fig_proto::prost::Message;
use fig_proto::remote::clientbound;
use fig_proto::remote::hostbound::ConfirmExchangeCredentialsRequest;
use fig_remote_ipc::figterm::{
    FigtermSessionId,
    FigtermState,
    SessionMetrics,
};
use fig_remote_ipc::AuthCode;
use parking_lot::Mutex;
use rand::distributions::uniform::SampleRange;
use time::OffsetDateTime;
use tokio::time::Instant;
use tracing::{
    debug,
    error,
    warn,
};

use crate::event::{
    EmitEventName,
    Event,
    WindowEvent,
};
use crate::platform::PlatformBoundEvent;
use crate::webview::notification::WebviewNotificationsState;
use crate::{
    EventLoopProxy,
    AUTOCOMPLETE_ID,
};

static LAST_EXECUTED_TIME: Mutex<SystemTime> = Mutex::new(SystemTime::UNIX_EPOCH);

#[derive(Debug, Clone)]
pub struct RemoteHook {
    pub notifications_state: Arc<WebviewNotificationsState>,
    pub proxy: EventLoopProxy,
}

#[async_trait::async_trait]
impl fig_remote_ipc::RemoteHookHandler for RemoteHook {
    async fn edit_buffer(
        &mut self,
        hook: &EditBufferHook,
        session_id: &FigtermSessionId,
        figterm_state: &Arc<FigtermState>,
        // notifications_state: &WebviewNotificationsState,
        // proxy: &EventLoopProxy,
    ) -> Result<Option<clientbound::response::Response>> {
        let _old_metrics = figterm_state.with_update(session_id.clone(), |session| {
            session.edit_buffer.text = hook.text.clone();
            session.edit_buffer.cursor = hook.cursor;
            session.terminal_cursor_coordinates = hook.terminal_cursor_coordinates.clone();
            session.context = hook.context.clone();

            let received_at = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
            let current_session_expired = session
                .current_session_metrics
                .as_ref()
                .map_or(false, |metrics| received_at > metrics.end_time + Duration::from_secs(5));

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

        // if let Some(metrics) = old_metrics.flatten() {
        //     if metrics.end_time > metrics.start_time {
        //         let properties: Vec<(&str, serde_json::Value)> = vec![
        //             ("start_time", metrics.start_time.format(&Rfc3339)?.into()),
        //             ("end_time", metrics.end_time.format(&Rfc3339)?.into()),
        //             (
        //                 "duration",
        //                 (metrics.end_time - metrics.start_time).whole_seconds().into(),
        //             ),
        //             ("num_insertions", metrics.num_insertions.into()),
        //             ("num_popups", metrics.num_popups.into()),
        //         ];
        //         //tokio::spawn(async {
        //             if let Err(err) = fig_telemetry::emit_track(fig_telemetry::TrackEvent::new(
        //                 fig_telemetry::TrackEventType::TerminalSessionMetricsRecorded,
        //                 fig_telemetry::TrackSource::Desktop,
        //                 env!("CARGO_PKG_VERSION").into(),
        //                 properties,
        //             ))
        //             .await
        //             {
        //                 warn!(%err, "Failed to record terminal session metrics");
        //             }
        //         });
        //     }
        // }

        let utf16_cursor_position = hook
            .text
            .get(..hook.cursor as usize)
            .map(|s| s.encode_utf16().count() as i32);

        for sub in self.notifications_state.subscriptions.iter() {
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
                            session_id: Some(session_id.into_string()),
                        },
                    )),
                })),
            };

            let mut encoded = BytesMut::new();
            message.encode(&mut encoded).unwrap();

            debug!(%message_id, "Sending edit buffer change notification to webview");

            self.proxy
                .send_event(Event::WindowEvent {
                    window_id: sub.key().clone(),
                    window_event: WindowEvent::Emit {
                        event_name: EmitEventName::Notification,
                        payload: BASE64_STANDARD.encode(encoded).into(),
                    },
                })
                .unwrap();
        }

        let empty_edit_buffer = hook.text.trim().is_empty();

        if !empty_edit_buffer {
            self.proxy
                .send_event(Event::PlatformBoundEvent(PlatformBoundEvent::EditBufferChanged))?;
        }

        self.proxy.send_event(Event::WindowEvent {
            window_id: AUTOCOMPLETE_ID,
            // If editbuffer is empty, hide the autocomplete window to avoid flickering
            window_event: if empty_edit_buffer {
                WindowEvent::Hide
            } else {
                WindowEvent::Show
            },
        })?;

        Ok(None)
    }

    async fn prompt(
        &mut self,
        hook: &PromptHook,
        session_id: &FigtermSessionId,
        figterm_state: &Arc<FigtermState>,
    ) -> Result<Option<clientbound::response::Response>> {
        let mut cwd_changed = false;
        let mut new_cwd = None;
        figterm_state.with(session_id, |session| {
            if let (Some(old_context), Some(new_context)) = (&session.context, &hook.context) {
                cwd_changed = old_context.current_working_directory != new_context.current_working_directory;
                new_cwd = new_context.current_working_directory.clone();
            }

            session.context = hook.context.clone();
        });

        if cwd_changed {
            if let Err(err) = self
                .notifications_state
                .broadcast_notification_all(
                    &NotificationType::NotifyOnLocationChange,
                    Notification {
                        r#type: Some(fig_proto::fig::notification::Type::LocationChangedNotification(
                            LocationChangedNotification {
                                session_id: Some(session_id.to_string()),
                                host_name: hook.context.as_ref().and_then(|ctx| ctx.hostname.clone()),
                                user_name: None,
                                directory: new_cwd,
                            },
                        )),
                    },
                    &self.proxy,
                )
                .await
            {
                error!(%err, "Failed to broadcast LocationChangedNotification");
            }
        }

        if let Err(err) = self
            .notifications_state
            .broadcast_notification_all(
                &NotificationType::NotifyOnPrompt,
                Notification {
                    r#type: Some(fig_proto::fig::notification::Type::ShellPromptReturnedNotification(
                        ShellPromptReturnedNotification {
                            session_id: Some(session_id.to_string()),
                            shell: hook.context.as_ref().map(|ctx| Process {
                                pid: ctx.pid,
                                executable: ctx.process_name.clone(),
                                directory: ctx.current_working_directory.clone(),
                                env: vec![],
                            }),
                        },
                    )),
                },
                &self.proxy,
            )
            .await
        {
            error!(%err, "Failed to broadcast ShellPromptReturnedNotification");
        }

        Ok(None)
    }

    async fn pre_exec(
        &mut self,
        hook: &PreExecHook,
        session_id: &FigtermSessionId,
        figterm_state: &Arc<FigtermState>,
    ) -> Result<Option<clientbound::response::Response>> {
        figterm_state.with_update(session_id.clone(), |session| {
            session.context = hook.context.clone();
        });

        self.proxy.send_event(Event::WindowEvent {
            window_id: AUTOCOMPLETE_ID.clone(),
            window_event: WindowEvent::Hide,
        })?;

        self.notifications_state
            .broadcast_notification_all(
                &NotificationType::NotifyOnProcessChanged,
                Notification {
                    r#type: Some(fig_proto::fig::notification::Type::ProcessChangeNotification(
                        ProcessChangedNotification {
                        session_id: Some(session_id.to_string()),
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
                &self.proxy,
            )
            .await?;

        Ok(None)
    }

    async fn post_exec(
        &mut self,
        hook: &PostExecHook,
        session_id: &FigtermSessionId,
        figterm_state: &Arc<FigtermState>,
    ) -> Result<Option<clientbound::response::Response>> {
        figterm_state.with_update(session_id.clone(), |session| {
            session.context = hook.context.clone();
        });

        self.notifications_state
            .broadcast_notification_all(
                &NotificationType::NotifyOnHistoryUpdated,
                Notification {
                    r#type: Some(fig_proto::fig::notification::Type::HistoryUpdatedNotification(
                        HistoryUpdatedNotification {
                            command: hook.command.clone(),
                            process_name: hook.context.as_ref().and_then(|ctx| ctx.process_name.clone()),
                            current_working_directory: hook
                                .context
                                .as_ref()
                                .and_then(|ctx| ctx.current_working_directory.clone()),
                            session_id: Some(session_id.to_string()),
                            hostname: hook.context.as_ref().and_then(|ctx| ctx.hostname.clone()),
                            exit_code: hook.exit_code,
                        },
                    )),
                },
                &self.proxy,
            )
            .await?;

        Ok(None)
    }

    async fn intercepted_key(
        &mut self,
        InterceptedKeyHook { action, context, .. }: InterceptedKeyHook,
    ) -> Result<Option<clientbound::response::Response>> {
        debug!(%action, "Intercepted Key Action");

        self.notifications_state
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
                &self.proxy,
            )
            .await?;

        Ok(None)
    }

    async fn account_info(&mut self) -> Result<Option<clientbound::response::Response>> {
        Err(anyhow!("account info not implemented"))
    }

    async fn start_exchange_credentials(
        &mut self,
        last_auth_code: &mut AuthCode,
    ) -> Result<Option<clientbound::response::Response>> {
        {
            let mut last_time = LAST_EXECUTED_TIME.lock();
            if last_time.elapsed().unwrap_or_default() < Duration::from_secs(1) {
                warn!("start_exchange_credentials hit rate limit");
                return Ok(None);
            }
            *last_time = SystemTime::now();
        }

        let new_code = (0..99999999).sample_single(&mut rand::thread_rng());
        *last_auth_code = Some((new_code, Instant::now()));

        if self
            .proxy
            .send_event(Event::ShowMessageNotification {
                title: "Credential exchange requested".into(),
                body: format!("Your exchange code is: {new_code:08}").into(),
                parent: None,
            })
            .is_err()
        {
            error!("event loop closed!");
        }

        Ok(None)
    }

    async fn confirm_exchange_credentials(
        &mut self,
        _request: ConfirmExchangeCredentialsRequest,
        _last_auth_code: &mut AuthCode,
    ) -> Result<Option<clientbound::response::Response>> {
        Err(anyhow::anyhow!("confirm_exchange_credentials not implemented"))
    }
}
