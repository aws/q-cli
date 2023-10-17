use std::sync::Arc;

use fig_proto::fig::notification::Type as NotificationEnum;
use fig_proto::fig::{
    NotificationType,
    SettingsChangedNotification,
};
use fig_settings::JsonStore;
use fig_util::directories;
use notify::{
    RecursiveMode,
    Watcher,
};
use serde_json::{
    Map,
    Value,
};
use tracing::{
    debug,
    error,
    trace,
};

use crate::notification_bus::NOTIFICATION_BUS;
use crate::webview::notification::WebviewNotificationsState;
use crate::EventLoopProxy;

pub async fn user_data_listener(notifications_state: Arc<WebviewNotificationsState>, proxy: EventLoopProxy) {
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

    let settings_path = match directories::settings_path().ok() {
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

    tokio::spawn(async move {
        let _watcher = watcher;
        while let Some(event) = rx.recv().await {
            trace!(?event, "Settings event");

            if let Some(ref settings_path) = settings_path {
                if event.paths.contains(settings_path) {
                    if let notify::EventKind::Create(_) | notify::EventKind::Modify(_) = event.kind {
                        match fig_settings::Settings::load_from_file() {
                            Ok(settings) => {
                                notifications_state
                                    .broadcast_notification_all(
                                        &NotificationType::NotifyOnSettingsChange,
                                        fig_proto::fig::Notification {
                                            r#type: Some(NotificationEnum::SettingsChangedNotification(
                                                SettingsChangedNotification {
                                                    json_blob: serde_json::to_string(&settings).ok(),
                                                },
                                            )),
                                        },
                                        &proxy,
                                    )
                                    .await
                                    .unwrap();

                                let mut mem_settings = fig_settings::Settings::load().expect("Failed to load state");

                                json_map_diff(
                                    &mem_settings.map(),
                                    &settings,
                                    |key, value| {
                                        debug!(%key, %value, "Setting added");
                                        NOTIFICATION_BUS.send_settings_new(key, value);
                                    },
                                    |key, old, new| {
                                        debug!(%key, %old, %new, "Setting change");
                                        NOTIFICATION_BUS.send_settings_changed(key, old, new);
                                    },
                                    |key, value| {
                                        debug!(%key, %value, "Setting removed");
                                        NOTIFICATION_BUS.send_settings_remove(key, value);
                                    },
                                );

                                *mem_settings.map_mut() = settings;
                            },
                            Err(err) => error!(%err, "Failed to get settings"),
                        }
                    }
                }
            }
        }
    });
}

// Diffs the old and new settings and calls the appropriate callbacks
fn json_map_diff(
    map_a: &Map<String, Value>,
    map_b: &Map<String, Value>,
    on_new: impl Fn(&str, &Value),
    on_changed: impl Fn(&str, &Value, &Value),
    on_removed: impl Fn(&str, &Value),
) {
    for (key, value) in map_a {
        if let Some(other_value) = map_b.get(key) {
            if value != other_value {
                on_changed(key, value, other_value);
            }
        } else {
            on_removed(key, value);
        }
    }

    for (key, value) in map_b {
        if !map_a.contains_key(key) {
            on_new(key, value);
        }
    }
}
