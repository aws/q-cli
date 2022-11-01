use std::sync::Arc;

use eyre::eyre;
use fig_ipc::local::send_hook_to_socket;
use fig_proto::hooks;
use fig_proto::local::file_changed_hook::FileChanged;
use fig_telemetry::{
    TrackEvent,
    TrackEventType,
    TrackSource,
};
use fig_util::directories;
use notify::{
    RecursiveMode,
    Watcher,
};
use parking_lot::RwLock;
use tokio::task::JoinHandle;
use tracing::{
    error,
    info,
    trace,
};

use super::DaemonStatus;

pub async fn spawn_settings_watcher(daemon_status: Arc<RwLock<DaemonStatus>>) -> JoinHandle<()> {
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

    let fig_app_bundle = fig_util::fig_bundle();

    match &fig_app_bundle {
        Some(app_bundle_path) if app_bundle_path.exists() => {
            match watcher.watch(app_bundle_path, RecursiveMode::NonRecursive) {
                Ok(()) => trace!("watching bundle at {app_bundle_path:?}"),
                Err(err) => {
                    error!(%err, "failed to watch application path dir");
                    daemon_status.write().settings_watcher_status =
                        Err(eyre!("Failed to watch application path dir\n{err}"));
                },
            }
        },
        _ => (),
    }

    let settings_path = match directories::settings_path().ok() {
        Some(settings_path) => match settings_path.parent() {
            Some(settings_dir) => {
                if let Err(err) = std::fs::create_dir_all(settings_dir) {
                    error!(%err, "failed to create settings dir");
                }
                match watcher.watch(settings_dir, RecursiveMode::NonRecursive) {
                    Ok(()) => {
                        trace!("watching settings file at {settings_dir:?}");
                        Some(settings_path)
                    },
                    Err(err) => {
                        error!(%err, "failed to watch settings dir");
                        daemon_status.write().settings_watcher_status =
                            Err(eyre!("Failed to watch settings dir\n{err}"));
                        None
                    },
                }
            },
            None => {
                error!("failed to get settings file dir");
                daemon_status.write().settings_watcher_status = Err(eyre!("Failed to get settings dir"));
                None
            },
        },
        None => {
            error!("failed to get settings file path");
            daemon_status.write().settings_watcher_status = Err(eyre!("No settings path"));
            None
        },
    };

    let state_path = match directories::state_path().ok() {
        Some(state_path) => match state_path.parent() {
            Some(state_dir) => {
                if let Err(err) = std::fs::create_dir_all(state_dir) {
                    error!(%err, "failed to create state dir");
                }
                match watcher.watch(state_dir, RecursiveMode::NonRecursive) {
                    Ok(()) => {
                        trace!("watching state dir at {state_dir:?}");
                        Some(state_path)
                    },
                    Err(err) => {
                        error!(%err, "failed to watch state dir");
                        daemon_status.write().settings_watcher_status = Err(eyre!("Failed to watch state dir\n{err}"));
                        None
                    },
                }
            },
            None => {
                error!("failed to get state file dir");
                daemon_status.write().settings_watcher_status = Err(eyre!("Failed to get state dir"));
                None
            },
        },
        None => {
            error!("failed to get state file path");
            daemon_status.write().settings_watcher_status = Err(eyre!("No state path"));
            None
        },
    };

    tokio::spawn(async move {
        let _watcher = watcher;
        while let Some(event) = rx.recv().await {
            trace!(?event, "Settings event");

            if let Some(ref settings_path) = settings_path {
                if event.paths.contains(settings_path) {
                    info!("Settings file changed");
                    let hook = hooks::new_file_changed_hook(
                        FileChanged::Settings,
                        settings_path.as_path().display().to_string(),
                    );
                    match send_hook_to_socket(hook.clone()).await {
                        Ok(()) => {
                            info!("Sent settings hook to daemon");
                            daemon_status.write().settings_watcher_status = Ok(());
                        },
                        Err(err) => {
                            error!("Failed to send hook: {err}");
                            daemon_status.write().settings_watcher_status = Err(eyre!(
                                "Failed to send settings hook to desktop app, is Fig running?\n{err}"
                            ));
                        },
                    }
                }
            }

            if let Some(ref state_path) = state_path {
                if event.paths.contains(state_path) {
                    info!("State file changed");
                    let hook =
                        hooks::new_file_changed_hook(FileChanged::State, state_path.as_path().display().to_string());
                    match send_hook_to_socket(hook.clone()).await {
                        Ok(_) => {
                            info!("Sent state hook to daemon");
                            daemon_status.write().settings_watcher_status = Ok(());
                        },
                        Err(err) => {
                            error!("Failed to send hook: {err}");
                            daemon_status.write().settings_watcher_status = Err(eyre!(
                                "Failed to send state hook to desktop app, is Fig running?\n{err}"
                            ));
                        },
                    }
                }
            }

            match &fig_app_bundle {
                Some(app_bundle_path) if event.paths.contains(app_bundle_path) => {
                    info!("application path changed");

                    // Do not run install logic on updates! Make sure update.lock is set in fig_update...
                    let update_lock = fig_util::directories::update_lock_path().ok();

                    match update_lock {
                        Some(file) if file.exists() => continue,
                        _ => (),
                    }

                    if !app_bundle_path.exists() {
                        fig_telemetry::emit_track(TrackEvent::new(
                            TrackEventType::UninstalledApp,
                            TrackSource::Daemon,
                            env!("CARGO_PKG_VERSION").into(),
                            [("source", "daemon settings watcher")],
                        ))
                        .await
                        .ok();

                        let url = fig_install::get_uninstall_url();
                        fig_util::open_url(url).ok();

                        fig_install::uninstall(fig_install::InstallComponents::all()).await.ok();
                    }
                },
                _ => (),
            }
        }
    })
}
