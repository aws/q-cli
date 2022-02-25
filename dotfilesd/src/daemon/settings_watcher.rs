use std::{sync::Arc, time::Duration};

use anyhow::{anyhow, Result};
use fig_ipc::hook::send_settings_changed;
use notify::{watcher, RecursiveMode, Watcher};
use parking_lot::RwLock;
use tracing::{error, info};

use super::DaemonStatus;

pub async fn spawn_settings_watcher(daemon_status: Arc<RwLock<DaemonStatus>>) -> Result<()> {
    // We need to spawn both a thread and a tokio task since the notify library does not
    // currently support async, this should be improved in the future, but currently this works fine

    let settings_path = fig_settings::settings::settings_path()?;
    let state_path = fig_settings::state::state_path()?;

    let (settings_watcher_tx, settings_watcher_rx) = std::sync::mpsc::channel();
    let mut watcher = watcher(settings_watcher_tx, Duration::from_secs(1))?;

    let (forward_tx, forward_rx) = flume::unbounded();

    tokio::task::spawn(async move {
        loop {
            match forward_rx.recv_async().await {
                Ok(event) => match send_settings_changed().await {
                    Ok(_) => {
                        info!("Settings changed: {:?}", event);
                        daemon_status.write().settings_watcher_status = Ok(());
                    }
                    Err(err) => {
                        error!("Could not send settings changed: {}", err);
                        daemon_status.write().settings_watcher_status = Err(err);
                    }
                },
                Err(err) => {
                    error!("Error while receiving settings: {}", err);
                    daemon_status.write().settings_watcher_status = Err(anyhow!(err));
                }
            }
        }
    });

    std::thread::spawn(move || {
        if let Err(err) = watcher.watch(&settings_path, RecursiveMode::NonRecursive) {
            error!("Could not watch {:?}: {}", settings_path, err);
        }
        if let Err(err) = watcher.watch(&state_path, RecursiveMode::NonRecursive) {
            error!("Could not watch {:?}: {}", state_path, err);
        }

        loop {
            match settings_watcher_rx.recv() {
                Ok(event) => {
                    if let Err(e) = forward_tx.send(event) {
                        error!("Error forwarding settings event: {}", e);
                    }
                }
                Err(err) => {
                    error!("Settings watcher rx: {}", err);
                }
            }
        }
    });

    Ok(())
}
