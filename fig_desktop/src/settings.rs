use std::sync::Arc;
use std::time::Duration;

use fig_proto::fig::notification::Type as NotificationEnum;
use fig_proto::fig::{
    LocalStateChangedNotification,
    NotificationType,
    SettingsChangedNotification,
};
use notify::{
    RecursiveMode,
    Watcher,
};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde_json::{
    Map,
    Value,
};
use tokio::fs::read_to_string;
use tracing::{
    error,
    trace,
};

use crate::notification::NotificationsState;
use crate::EventLoopProxy;

static SETTINGS: Lazy<Mutex<Map<String, Value>>> =
    Lazy::new(|| Mutex::new(fig_settings::settings::get_map().unwrap_or_default()));

static STATE: Lazy<Mutex<Map<String, Value>>> =
    Lazy::new(|| Mutex::new(fig_settings::state::get_map().unwrap_or_default()));

pub async fn settings_listener(notifications_state: Arc<NotificationsState>, proxy: EventLoopProxy) {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    let mut watcher = notify::recommended_watcher(move |res| match res {
        Ok(event) => {
            if let Err(err) = tx.send(event) {
                error!(%err, "failed to send notify event")
            }
        },
        Err(err) => error!(%err, "notify watcher"),
    })
    .unwrap();

    // watcher.configure(notify::Config::PreciseEvents(true)).unwrap();
    // watcher.configure(notify::Config::NoticeEvents(true)).unwrap();
    watcher
        .configure(notify::Config::OngoingEvents(Some(Duration::from_secs_f32(2.25))))
        .unwrap();

    let settings_path = match fig_settings::settings::settings_path().ok() {
        Some(settings_path) => match settings_path.parent() {
            Some(settings_dir) => match watcher.watch(settings_dir, RecursiveMode::NonRecursive) {
                Ok(()) => {
                    trace!("watching settings file at {settings_dir:?}");
                    Some(settings_path)
                },
                Err(err) => {
                    error!(%err, "failed to watch settings dir");
                    None
                },
            },
            None => {
                error!("failed to get settings file dir");
                None
            },
        },
        None => {
            error!("failed to get settings file path");
            None
        },
    };

    let state_path = match fig_settings::state::state_path().ok() {
        Some(state_path) => match state_path.parent() {
            Some(state_dir) => match watcher.watch(state_dir, RecursiveMode::NonRecursive) {
                Ok(()) => {
                    trace!("watching state dir at {state_dir:?}");
                    Some(state_path)
                },
                Err(err) => {
                    error!(%err, "failed to watch state dir");
                    None
                },
            },
            None => {
                error!("failed to get state file dir");
                None
            },
        },
        None => {
            error!("failed to get state file path");
            None
        },
    };

    tokio::spawn(async move {
        let _watcher = watcher;
        while let Some(event) = rx.recv().await {
            trace!(?event, "Settings event");

            if let Some(ref settings_path) = settings_path {
                if event.paths.contains(settings_path) {
                    if let notify::EventKind::Create(_) | notify::EventKind::Modify(_) = event.kind {
                        let settings_str = read_to_string(settings_path).await.ok();

                        if let Some(settings_str) = &settings_str {
                            if let Ok(settings_map) = serde_json::from_str(settings_str) {
                                *SETTINGS.lock() = settings_map;
                            }
                        }

                        notifications_state
                            .send_notification(
                                &NotificationType::NotifyOnSettingsChange,
                                fig_proto::fig::Notification {
                                    r#type: Some(NotificationEnum::SettingsChangedNotification(
                                        SettingsChangedNotification {
                                            json_blob: settings_str,
                                        },
                                    )),
                                },
                                &proxy,
                            )
                            .await
                            .unwrap();
                    }
                }
            }

            if let Some(ref state_path) = state_path {
                if event.paths.contains(state_path) {
                    if let notify::EventKind::Create(_) | notify::EventKind::Modify(_) = event.kind {
                        let state_str = read_to_string(state_path).await.ok();

                        if let Some(state_str) = &state_str {
                            if let Ok(state_map) = serde_json::from_str(state_str) {
                                *STATE.lock() = state_map;
                            }
                        }

                        notifications_state
                            .send_notification(
                                &NotificationType::NotifyOnLocalStateChanged,
                                fig_proto::fig::Notification {
                                    r#type: Some(NotificationEnum::LocalStateChangedNotification(
                                        LocalStateChangedNotification { json_blob: state_str },
                                    )),
                                },
                                &proxy,
                            )
                            .await
                            .unwrap();
                    }
                }
            }
        }
    });
}
