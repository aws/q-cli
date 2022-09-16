use std::sync::Arc;
use std::time::Duration;

use eyre::eyre;
use fig_ipc::local::send_hook_to_socket;
use fig_proto::hooks;
use fig_proto::local::file_changed_hook::FileChanged;
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
use crate::cli::app::uninstall::UninstallArgs;
use crate::util::fig_bundle;

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

    watcher
        .configure(notify::Config::OngoingEvents(Some(Duration::from_secs_f32(0.25))))
        .unwrap();

    let application_path = std::path::Path::new("/Applications/Fig.app");
    let application_path_clone = std::path::PathBuf::from(application_path);

    if application_path.exists() {
        match watcher.watch(application_path_clone.as_path(), RecursiveMode::NonRecursive) {
            Ok(()) => trace!("watching bundle at {application_path:?}"),
            Err(err) => {
                error!(%err, "failed to watch application path dir");
                daemon_status.write().settings_watcher_status = Err(eyre!(err));
            },
        }
    }

    let settings_path = match fig_settings::settings::settings_path().ok() {
        Some(settings_path) => match settings_path.parent() {
            Some(settings_dir) => {
                if let Err(err) = std::fs::create_dir_all(&settings_dir) {
                    error!(%err, "failed to create settings dir");
                }
                match watcher.watch(settings_dir, RecursiveMode::NonRecursive) {
                    Ok(()) => {
                        trace!("watching settings file at {settings_dir:?}");
                        Some(settings_path)
                    },
                    Err(err) => {
                        error!(%err, "failed to watch settings dir");
                        daemon_status.write().settings_watcher_status = Err(eyre!(err));
                        None
                    },
                }
            },
            None => {
                error!("failed to get settings file dir");
                daemon_status.write().settings_watcher_status = Err(eyre!("failed to get settings dir"));
                None
            },
        },
        None => {
            error!("failed to get settings file path");
            daemon_status.write().settings_watcher_status = Err(eyre!("no settings path"));
            None
        },
    };

    let state_path = match fig_settings::state::state_path().ok() {
        Some(state_path) => match state_path.parent() {
            Some(state_dir) => {
                if let Err(err) = std::fs::create_dir_all(&state_dir) {
                    error!(%err, "failed to create state dir");
                }
                match watcher.watch(state_dir, RecursiveMode::NonRecursive) {
                    Ok(()) => {
                        trace!("watching state dir at {state_dir:?}");
                        Some(state_path)
                    },
                    Err(err) => {
                        error!(%err, "failed to watch state dir");
                        daemon_status.write().settings_watcher_status = Err(eyre!(err));
                        None
                    },
                }
            },
            None => {
                error!("failed to get state file dir");
                daemon_status.write().settings_watcher_status = Err(eyre!("failed to get state dir"));
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
                    if let Err(err) = send_hook_to_socket(hook.clone()).await {
                        error!("Failed to send hook: {err}");
                        daemon_status.write().settings_watcher_status = Err(eyre!(err));
                    }
                }
            }

            if let Some(ref state_path) = state_path {
                if event.paths.contains(state_path) {
                    info!("State file changed");
                    let hook =
                        hooks::new_file_changed_hook(FileChanged::State, state_path.as_path().display().to_string());
                    if let Err(err) = send_hook_to_socket(hook.clone()).await {
                        error!("Failed to send hook: {err}");
                        daemon_status.write().settings_watcher_status = Err(eyre!(err));
                    }
                }
            }

            if event.paths.contains(&application_path_clone) {
                info!("Application path changed");

                tokio::time::sleep(Duration::from_secs(1)).await;

                if let Some(app_bundle_exists) = fig_bundle() {
                    if !app_bundle_exists.is_dir() {
                        crate::cli::app::uninstall::uninstall_mac_app(&UninstallArgs {
                            user_data: true,
                            app_bundle: true,
                            input_method: true,
                            terminal_integrations: true,
                            daemon: true,
                            dotfiles: true,
                            ssh: true,
                            no_open: false,
                        })
                        .await;
                    }
                }
            }
        }
    })
}
