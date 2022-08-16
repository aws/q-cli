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
use tokio::fs::read_to_string;
use tracing::{
    error,
    trace,
};

use crate::notification::NotificationsState;
use crate::EventLoopProxy;

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
                        notifications_state
                            .send_notification(
                                &NotificationType::NotifyOnSettingsChange,
                                fig_proto::fig::Notification {
                                    r#type: Some(NotificationEnum::SettingsChangedNotification(
                                        SettingsChangedNotification {
                                            json_blob: read_to_string(settings_path).await.ok(),
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
                        notifications_state
                            .send_notification(
                                &NotificationType::NotifyOnLocalStateChanged,
                                fig_proto::fig::Notification {
                                    r#type: Some(NotificationEnum::LocalStateChangedNotification(
                                        LocalStateChangedNotification {
                                            json_blob: read_to_string(state_path).await.ok(),
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
        }
    });
}
