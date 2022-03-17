use std::{sync::Arc, time::Duration};

use crate::util::fig_bundle;
use anyhow::{anyhow, Context, Result};
use fig_ipc::hook::send_hook_to_socket;
use fig_proto::{hooks, local::file_changed_hook::FileChanged};
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use parking_lot::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, error, info};

use super::DaemonStatus;

pub async fn spawn_settings_watcher(
    daemon_status: Arc<RwLock<DaemonStatus>>,
) -> Result<JoinHandle<()>> {
    // We need to spawn both a thread and a tokio task since the notify library does not
    // currently support async, this should be improved in the future, but currently this works fine

    let settings_path =
        fig_settings::settings::settings_path().context("Could not get settings path")?;
    let state_path = fig_settings::state::state_path().context("Could not get state path")?;
    let application_path = "/Applications/Fig.app";

    let (settings_watcher_tx, settings_watcher_rx) = std::sync::mpsc::channel();
    let mut watcher = watcher(settings_watcher_tx, Duration::from_secs(1))?;

    let (forward_tx, forward_rx) = flume::unbounded();

    let settings_path_clone = settings_path.clone();
    let state_path_clone = state_path.clone();
    let application_path_clone = std::path::PathBuf::from(application_path);

    let daemon_status_clone = daemon_status.clone();
    let tokio_join = tokio::task::spawn(async move {
        let daemon_status = daemon_status_clone;
        loop {
            match forward_rx.recv_async().await {
                Ok(event) => {
                    debug!("Received event: {:?}", event);

                    match event {
                        DebouncedEvent::NoticeWrite(path) | DebouncedEvent::NoticeRemove(path) => {
                            match path {
                                path if path == settings_path_clone.as_path() => {
                                    info!("Settings file changed");
                                    let hook = hooks::new_file_changed_hook(
                                        FileChanged::Settings,
                                        settings_path_clone.as_path().display().to_string(),
                                    );
                                    if let Err(err) = send_hook_to_socket(hook.clone()).await {
                                        error!("Failed to send hook: {:?}", err);
                                    }
                                }
                                path if path == state_path_clone.as_path() => {
                                    info!("State file changed");
                                    let hook = hooks::new_file_changed_hook(
                                        FileChanged::State,
                                        state_path_clone.as_path().display().to_string(),
                                    );
                                    if let Err(err) = send_hook_to_socket(hook.clone()).await {
                                        error!("Failed to send hook: {:?}", err);
                                    }
                                }
                                path if path == application_path_clone.as_path() => {
                                    info!("Application path changed");

                                    tokio::time::sleep(Duration::from_secs(1)).await;

                                    let app_bundle_exists = fig_bundle().unwrap().is_dir();

                                    if !app_bundle_exists {
                                        crate::cli::app::uninstall::uninstall_mac_app().await;
                                    }
                                }
                                unknown_path => {
                                    error!("Unknown path changed: {:?}", unknown_path);
                                }
                            }
                        }
                        DebouncedEvent::Error(err, path) => {
                            error!("Error watching settings ({:?}): {:?}", path, err);
                            daemon_status.write().settings_watcher_status = Err(anyhow!(err));
                        }
                        event => {
                            debug!("Ignoring event: {:?}", event);
                        }
                    }
                }
                Err(err) => {
                    daemon_status.write().settings_watcher_status = Err(anyhow!(err));
                    break;
                }
            }
        }
    });

    std::thread::spawn(move || {
        let settings_watcher_rx = settings_watcher_rx;

        if let Err(err) = watcher.watch(&*settings_path, RecursiveMode::NonRecursive) {
            error!("Could not watch {:?}: {}", settings_path, err);
            daemon_status.write().settings_watcher_status = Err(anyhow!(err));
        }
        if let Err(err) = watcher.watch(&*state_path, RecursiveMode::NonRecursive) {
            error!("Could not watch {:?}: {}", state_path, err);
            daemon_status.write().settings_watcher_status = Err(anyhow!(err));
        }

        if let Err(err) = watcher.watch(&*application_path, RecursiveMode::NonRecursive) {
            error!("Could not watch {:?}: {}", application_path, err);
            daemon_status.write().settings_watcher_status = Err(anyhow!(err));
        }

        loop {
            match settings_watcher_rx.recv() {
                Ok(event) => {
                    if let Err(err) = forward_tx.send(event) {
                        error!("Error forwarding settings event: {}", err);
                        daemon_status.write().settings_watcher_status = Err(anyhow!(err));
                    }
                }
                Err(err) => {
                    error!("Settings watcher rx: {}", err);
                    daemon_status.write().settings_watcher_status = Err(anyhow!(err));
                }
            }
        }
    });

    Ok(tokio_join)
}
