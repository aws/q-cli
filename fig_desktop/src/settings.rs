use notify::{
    RecursiveMode,
    Watcher,
};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use tracing::error;

static _SETTINGS: Lazy<Mutex<serde_json::Value>> = Lazy::new(|| Mutex::new(serde_json::Value::Null));
static _STATE: Lazy<Mutex<serde_json::Value>> = Lazy::new(|| Mutex::new(serde_json::Value::Null));

pub async fn _settings_listener() {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    let mut watcher = notify::recommended_watcher(move |res| match res {
        Ok(event) => tx.send(event).unwrap(),
        Err(err) => error!("notify watcher: {err:?}"),
    })
    .unwrap();

    if let Some(settings_path) = fig_settings::settings::settings_path() {
        watcher.watch(&settings_path, RecursiveMode::NonRecursive).unwrap();
    }

    if let Some(state_path) = fig_settings::state::state_path() {
        watcher.watch(&state_path, RecursiveMode::NonRecursive).unwrap();
    }

    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            tracing::info!("{event:?}");
        }
    });
}
