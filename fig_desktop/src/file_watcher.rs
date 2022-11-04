use std::sync::Arc;

use fig_proto::fig::notification::Type as NotificationEnum;
use fig_proto::fig::{
    LocalStateChangedNotification,
    NotificationType,
    SettingsChangedNotification,
};
use fig_request::auth::Credentials;
use fig_settings::JsonStore;
use fig_util::directories;
use notify::{
    RecursiveMode,
    Watcher,
};
use once_cell::sync::Lazy;
use serde_json::{
    Map,
    Value,
};
use tokio::sync::Mutex;
use tracing::{
    debug,
    error,
    trace,
};

use crate::event::Event;
use crate::notification_bus::NOTIFICATION_BUS;
use crate::webview::notification::WebviewNotificationsState;
use crate::EventLoopProxy;

static CREDENTIALS: Lazy<Mutex<Credentials>> =
    Lazy::new(|| Mutex::new(fig_request::auth::Credentials::load_credentials().unwrap_or_default()));

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

    let state_path = match directories::state_path().ok() {
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

    let credentials_path = match directories::credentials_path().ok() {
        Some(credentials_path) => match credentials_path.parent() {
            Some(credentials_dir) => match watcher.watch(credentials_dir, RecursiveMode::NonRecursive) {
                Ok(()) => {
                    trace!("watching credentials dir at {credentials_dir:?}");
                    Some(credentials_path)
                },
                Err(err) => {
                    error!(%err, "failed to watch credentials dir");
                    None
                },
            },
            None => {
                error!("failed to get credentials file dir");
                None
            },
        },
        None => {
            error!("failed to get credentials file path");
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

            if let Some(ref state_path) = state_path {
                if event.paths.contains(state_path) {
                    if let notify::EventKind::Create(_) | notify::EventKind::Modify(_) = event.kind {
                        match fig_settings::State::load_from_file() {
                            Ok(state) => {
                                notifications_state
                                    .broadcast_notification_all(
                                        &NotificationType::NotifyOnLocalStateChanged,
                                        fig_proto::fig::Notification {
                                            r#type: Some(NotificationEnum::LocalStateChangedNotification(
                                                LocalStateChangedNotification {
                                                    json_blob: serde_json::to_string(&state).ok(),
                                                },
                                            )),
                                        },
                                        &proxy,
                                    )
                                    .await
                                    .unwrap();

                                let mut mem_state = fig_settings::State::load().expect("Failed to load state");

                                json_map_diff(
                                    &mem_state.map(),
                                    &state,
                                    |key, value| {
                                        debug!(%key, %value, "State added");
                                        NOTIFICATION_BUS.send_state_new(key, value);
                                    },
                                    |key, old, new| {
                                        debug!(%key, %old, %new, "State change");
                                        NOTIFICATION_BUS.send_state_changed(key, old, new);
                                    },
                                    |key, value| {
                                        debug!(%key, %value, "State removed");
                                        NOTIFICATION_BUS.send_state_remove(key, value);
                                    },
                                );

                                *mem_state.map_mut() = state;
                            },
                            Err(err) => error!(%err, "Failed to get state"),
                        }
                    }
                }
            }

            if let Some(ref credentials_path) = credentials_path {
                if event.paths.contains(credentials_path) {
                    if let notify::EventKind::Create(_) | notify::EventKind::Modify(_) | notify::EventKind::Remove(_) =
                        event.kind
                    {
                        let creds = fig_request::auth::Credentials::load_credentials().unwrap_or_default();
                        if creds.email != CREDENTIALS.lock().await.email {
                            NOTIFICATION_BUS.send_user_email(creds.email.clone());
                            proxy.send_event(Event::ReloadCredentials).ok();
                        }
                        *CREDENTIALS.lock().await = creds;
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
