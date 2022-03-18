pub mod launchd_plist;
pub mod scheduler;
pub mod settings_watcher;
pub mod systemd_unit;
pub mod websocket;

use crate::{
    daemon::{
        launchd_plist::LaunchdPlist, settings_watcher::spawn_settings_watcher,
        systemd_unit::SystemdUnit, websocket::process_websocket,
    },
    util::{backoff::Backoff, launch_fig, LaunchOptions},
};

use anyhow::{anyhow, Context, Result};
use fig_ipc::{daemon::get_daemon_socket_path, recv_message, send_message};
use fig_proto::daemon::diagnostic_response::{
    settings_watcher_status, unix_socket_status, websocket_status, SettingsWatcherStatus,
    UnixSocketStatus, WebsocketStatus,
};
use futures::{SinkExt, StreamExt};
use parking_lot::RwLock;
use rand::{distributions::Uniform, prelude::Distribution};
use std::{
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
    time::Duration,
};
use tokio::{
    fs::remove_file,
    net::{UnixListener, UnixStream},
    select,
    task::JoinHandle,
};
use tokio_tungstenite::tungstenite;
use tracing::{debug, error, info, trace};

pub fn get_daemon() -> Result<LaunchService> {
    #[cfg(target_os = "macos")]
    {
        LaunchService::launchd()
    }
    #[cfg(target_os = "linux")]
    {
        LaunchService::systemd()
    }
    #[cfg(target_os = "windows")]
    {
        return Err(anyhow::anyhow!("Windows is not yet supported"));
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    return Err(anyhow::anyhow!("Unsupported platform"));
}

pub fn install_daemon() -> Result<()> {
    get_daemon()?.install()
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitSystem {
    /// macOS init system
    ///
    /// <https://launchd.info/>
    Launchd,
    /// Most common Linux init system
    ///
    /// <https://systemd.io/>
    Systemd,
    /// Init system used by artix, void, etc
    ///
    /// <http://smarden.org/runit/>
    Runit,
    /// Init subsystem used by alpine, gentoo, etc
    ///
    /// <https://wiki.gentoo.org/wiki/Project:OpenRC>
    OpenRc,
}

impl InitSystem {
    pub fn get_init_system() -> Result<InitSystem> {
        let output = Command::new("ps")
            .args(["1"])
            .output()
            .context("Could not run ps")?;

        if !output.status.success() {
            return Err(anyhow!("ps failed: {}", output.status));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        if stdout.contains("launchd") {
            Ok(InitSystem::Launchd)
        } else if stdout.contains("systemd") {
            Ok(InitSystem::Systemd)
        } else if stdout.contains("runit") {
            Ok(InitSystem::Runit)
        } else if stdout.contains("openrc") {
            Ok(InitSystem::OpenRc)
        } else {
            Err(anyhow!("Could not determine init system"))
        }
    }

    pub fn start_daemon(&self, path: impl AsRef<Path>) -> Result<()> {
        match self {
            InitSystem::Launchd => {
                let output = Command::new("launchctl")
                    .arg("load")
                    .arg(path.as_ref())
                    .output()?;

                if !output.status.success() {
                    return Err(anyhow!(
                        "Could not start daemon: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }

                let stderr = String::from_utf8_lossy(&output.stderr);

                if !stderr.is_empty() {
                    return Err(anyhow!(
                        "Could not start daemon: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }

                Ok(())
            }
            InitSystem::Systemd => {
                let output = Command::new("systemctl")
                    .arg("--now")
                    .arg("enable")
                    .arg(path.as_ref())
                    .output()
                    .with_context(|| format!("Could not enable {:?}", path.as_ref()))?;

                if !output.status.success() {
                    return Err(anyhow!(
                        "Could not start daemon: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }

                Ok(())
            }
            _ => Err(anyhow!("Could not start daemon: unsupported init system")),
        }
    }

    fn stop_daemon(&self, path: impl AsRef<Path>) -> Result<()> {
        match self {
            InitSystem::Launchd => {
                let output = Command::new("launchctl")
                    .arg("unload")
                    .arg(path.as_ref())
                    .output()?;

                if !output.status.success() {
                    return Err(anyhow!(
                        "Could not stop daemon: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }

                let stderr = String::from_utf8_lossy(&output.stderr);

                if !stderr.is_empty() {
                    return Err(anyhow!(
                        "Could not stop daemon: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }

                Ok(())
            }
            InitSystem::Systemd => {
                Command::new("systemctl")
                    .arg("--now")
                    .arg("disable")
                    .arg(path.as_ref())
                    .output()
                    .with_context(|| format!("Could not disable {:?}", path.as_ref()))?;

                Ok(())
            }
            _ => Err(anyhow!("Could not stop daemon: unsupported init system")),
        }
    }

    pub fn restart_daemon(&self, path: impl AsRef<Path>) -> Result<()> {
        self.stop_daemon(path.as_ref()).ok();
        self.start_daemon(path.as_ref())?;

        // TODO: use restart functionality of init system if possible

        Ok(())
    }

    pub fn daemon_name(&self) -> &'static str {
        match self {
            InitSystem::Launchd => "io.fig.dotfiles-daemon",
            InitSystem::Systemd => "fig-dotfiles-daemon",
            _ => unimplemented!(),
        }
    }

    pub fn daemon_status(&self) -> Result<Option<i32>> {
        match self {
            InitSystem::Launchd => {
                let output = Command::new("launchctl").arg("list").output()?;

                let stdout = String::from_utf8_lossy(&output.stdout);

                let status = stdout
                    .lines()
                    .map(|line| line.split_whitespace().collect::<Vec<_>>())
                    .find(|line| line[2] == self.daemon_name())
                    .and_then(|data| data[1].parse::<i32>().ok());

                Ok(status)
            }
            InitSystem::Systemd => Err(anyhow!("todo")),
            _ => Err(anyhow!(
                "Could not get daemon status: unsupported init system"
            )),
        }
    }
}

/// A service that can be launched by the init system
pub struct LaunchService {
    /// Path to the service's definition file
    pub path: PathBuf,
    /// The service's definition
    pub data: String,
    /// The init system to use
    pub launch_system: InitSystem,
}

impl LaunchService {
    pub fn launchd() -> Result<LaunchService> {
        let homedir = fig_directories::home_dir().context("Could not get home directory")?;

        let plist_path = homedir
            .join("Library")
            .join("LaunchAgents")
            .join("io.fig.dotfiles-daemon.plist");

        let executable_path = std::env::current_exe()?;
        let executable_path_str = executable_path.to_string_lossy();

        let log_path = homedir.join(".fig").join("logs").join("daemon.log");
        let log_path_str = log_path.to_string_lossy();

        let plist = LaunchdPlist::new(InitSystem::Launchd.daemon_name())
            .program(&*executable_path_str)
            .program_arguments([&*executable_path_str, "daemon"])
            .keep_alive(true)
            .run_at_load(true)
            .throttle_interval(20)
            .standard_out_path(&*log_path_str)
            .standard_error_path(&*log_path_str)
            .environment_variable("FIG_LOG_LEVEL", "debug")
            .plist();

        Ok(LaunchService {
            path: plist_path,
            data: plist,
            launch_system: InitSystem::Launchd,
        })
    }

    pub fn systemd() -> Result<LaunchService> {
        let homedir = fig_directories::home_dir().context("Could not get home directory")?;

        let path = homedir
            .join(".config")
            .join("systemd")
            .join("user")
            .join("fig-dotfiles-daemon.service");

        let executable_path = std::env::current_exe()?;
        let executable_path_str = executable_path.to_string_lossy();

        let unit = SystemdUnit::new("Fig Dotfiles Daemon")
            .exec_start(executable_path_str)
            .restart("always")
            .restart_sec(5)
            .wanted_by("default.target")
            .unit();

        Ok(LaunchService {
            path,
            data: unit,
            launch_system: InitSystem::Systemd,
        })
    }

    pub fn start(&self) -> Result<()> {
        self.launch_system.start_daemon(self.path.as_path())
    }

    pub fn stop(&self) -> Result<()> {
        self.launch_system.stop_daemon(self.path.as_path())
    }

    pub fn restart(&self) -> Result<()> {
        self.launch_system.restart_daemon(self.path.as_path())
    }

    pub fn install(&self) -> Result<()> {
        // Write to the definition file
        let mut file = std::fs::File::create(&self.path)?;
        file.write_all(self.data.as_bytes())?;
        self.restart()
    }

    pub fn uninstall(&self) -> Result<()> {
        self.stop().ok();

        if self.path.exists() {
            // Remove the definition file
            std::fs::remove_file(&self.path)?;
        }

        Ok(())
    }
}

pub struct DaemonStatus {
    /// The time the daemon was started as a u64 timestamp in seconds since the epoch
    time_started: u64,
    settings_watcher_status: Result<()>,
    websocket_status: Result<()>,
    unix_socket_status: Result<()>,
}

async fn spawn_unix_handler(
    mut stream: UnixStream,
    daemon_status: Arc<RwLock<DaemonStatus>>,
) -> Result<()> {
    tokio::task::spawn(async move {
        loop {
            match recv_message::<fig_proto::daemon::DaemonMessage, _>(&mut stream).await {
                Ok(Some(message)) => {
                    trace!("Received message: {:?}", message);

                    if let Some(command) = &message.command {
                        let response = match command {
                            fig_proto::daemon::daemon_message::Command::Diagnostic(
                                diagnostic_command,
                            ) => {
                                let parts: Vec<_> = diagnostic_command.parts().collect();

                                let daemon_status = daemon_status.read();

                                let time_started_epoch =
                                    (parts.is_empty() ||
                                        parts.contains(&fig_proto::daemon::diagnostic_command::DiagnosticPart::TimeStartedEpoch))
                                    .then(|| {
                                        daemon_status.time_started
                                });

                                let settings_watcher_status =
                                    (parts.is_empty() ||
                                        parts.contains(&fig_proto::daemon::diagnostic_command::DiagnosticPart::SettingsWatcherStatus))
                                    .then(|| {
                                        match &daemon_status.settings_watcher_status {
                                            Ok(_) => SettingsWatcherStatus {
                                                status: settings_watcher_status::Status::Ok.into(),
                                                error: None,
                                            },
                                            Err(err) => SettingsWatcherStatus {
                                                status: settings_watcher_status::Status::Error.into(),
                                                error: Some(err.to_string()),
                                            },
                                        }
                                });

                                let websocket_status =
                                    (parts.is_empty() ||
                                        parts.contains(&fig_proto::daemon::diagnostic_command::DiagnosticPart::WebsocketStatus))
                                    .then(|| {
                                        match &daemon_status.websocket_status {
                                            Ok(_) => WebsocketStatus {
                                                status: websocket_status::Status::Ok.into(),
                                                error: None,
                                            },
                                            Err(err) => WebsocketStatus {
                                                status: websocket_status::Status::Error.into(),
                                                error: Some(err.to_string()),
                                            },
                                        }
                                });

                                let unix_socket_status =
                                    (parts.is_empty() ||
                                        parts.contains(&fig_proto::daemon::diagnostic_command::DiagnosticPart::UnixSocketStatus))
                                    .then(|| {
                                        match &daemon_status.unix_socket_status {
                                            Ok(_) => UnixSocketStatus {
                                                status: unix_socket_status::Status::Ok.into(),
                                                error: None,
                                            },
                                            Err(err) => UnixSocketStatus {
                                                status: unix_socket_status::Status::Error.into(),
                                                error: Some(err.to_string()),
                                            },
                                        }
                                });

                                fig_proto::daemon::new_diagnostic_response(
                                    time_started_epoch,
                                    settings_watcher_status,
                                    websocket_status,
                                    unix_socket_status,
                                )
                            }
                            fig_proto::daemon::daemon_message::Command::SelfUpdate(_) => {
                                let success = match fig_ipc::command::update_command(true).await {
                                    Ok(()) => {
                                        tokio::task::spawn(async {
                                            tokio::time::sleep(std::time::Duration::from_secs(5))
                                                .await;

                                            tokio::task::block_in_place(|| {
                                                launch_fig(
                                                    LaunchOptions::new().wait_for_activation(),
                                                )
                                                .ok();
                                            });
                                        });
                                        true
                                    }
                                    Err(err) => {
                                        error!("Failed to update: {}", err);
                                        false
                                    }
                                };

                                fig_proto::daemon::new_self_update_response(success)
                            }
                        };

                        if !message.no_response() {
                            let response = fig_proto::daemon::DaemonResponse {
                                id: message.id,
                                response: Some(response),
                            };

                            if let Err(err) = send_message(&mut stream, response).await {
                                error!("Error sending message: {}", err);
                            }
                        }
                    }
                }
                Ok(None) => {
                    info!("Received EOF while reading message");
                    break;
                }
                Err(err) => {
                    error!("Error while receiving message: {}", err);
                    break;
                }
            }
        }
    });

    Ok(())
}

pub async fn spawn_incomming_unix_handler(
    daemon_status: Arc<RwLock<DaemonStatus>>,
) -> Result<JoinHandle<()>> {
    let unix_socket_path = get_daemon_socket_path();

    // Create the unix socket directory if it doesn't exist
    if let Some(unix_socket_dir) = unix_socket_path.parent() {
        tokio::fs::create_dir_all(unix_socket_dir)
            .await
            .context("Could not create unix socket directory")?;
    }

    // Remove the unix socket if it already exists
    if unix_socket_path.exists() {
        remove_file(&unix_socket_path).await?;
    }

    // Bind the unix socket
    let unix_socket =
        UnixListener::bind(&unix_socket_path).context("Could not connect to unix socket")?;

    Ok(tokio::spawn(async move {
        while let Ok((stream, _addr)) = unix_socket.accept().await {
            if let Err(err) = spawn_unix_handler(stream, daemon_status.clone()).await {
                error!(
                    "Error while spawining unix socket connection handler: {}",
                    err
                );
            }
        }
    }))
}

/// Spawn the daemon to listen for updates and dotfiles changes
pub async fn daemon() -> Result<()> {
    info!("Starting daemon...");

    let daemon_status = Arc::new(RwLock::new(DaemonStatus {
        time_started: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
        settings_watcher_status: Ok(()),
        websocket_status: Ok(()),
        unix_socket_status: Ok(()),
    }));

    // Add small random element to the delay to avoid all clients from sending the messages at the same time
    let dist = Uniform::new(59., 60.);
    let delay = dist.sample(&mut rand::thread_rng());
    let mut ping_interval = tokio::time::interval(Duration::from_secs_f64(delay));

    // Spawn task scheduler
    let (mut scheduler, scheduler_join) = scheduler::Scheduler::spawn().await;
    match fig_settings::state::get_value("dotfiles.all.lastUpdated")
        .ok()
        .flatten()
    {
        Some(_) => scheduler.schedule_random_delay(scheduler::SyncDotfiles, 60., 1260.),
        None => scheduler.schedule_random_delay(scheduler::SyncDotfiles, 0., 60.),
    }

    // Spawn the incoming handler
    let daemon_status_clone = daemon_status.clone();
    let unix_join = tokio::spawn(async move {
        let daemon_status = daemon_status_clone;
        let mut backoff =
            Backoff::new(Duration::from_secs_f64(0.25), Duration::from_secs_f64(120.));
        loop {
            match spawn_incomming_unix_handler(daemon_status.clone()).await {
                Ok(handle) => {
                    daemon_status.write().unix_socket_status = Ok(());
                    backoff.reset();
                    if let Err(err) = handle.await {
                        error!("Error on unix handler join: {:?}", err);
                        daemon_status.write().unix_socket_status = Err(err.into());
                    }
                    return;
                }
                Err(err) => {
                    error!("Error spawning unix handler: {:?}", err);
                    daemon_status.write().unix_socket_status = Err(err);
                }
            }
            backoff.sleep().await;
        }
    });

    // Spawn websocket handler
    let daemon_status_clone = daemon_status.clone();
    let websocket_join = tokio::spawn(async move {
        let daemon_status = daemon_status_clone;
        let mut backoff =
            Backoff::new(Duration::from_secs_f64(0.25), Duration::from_secs_f64(120.));
        loop {
            match websocket::connect_to_fig_websocket().await {
                Ok(mut websocket_stream) => {
                    daemon_status.write().websocket_status = Ok(());
                    backoff.reset();
                    loop {
                        select! {
                            next = websocket_stream.next() => {
                                match process_websocket(&next, &mut scheduler).await {
                                    Ok(()) => {}
                                    Err(err) => {
                                        error!("Error while processing websocket message: {}", err);
                                        daemon_status.write().websocket_status = Err(err);
                                        break;
                                    }
                                }
                            }
                            _ = ping_interval.tick() => {
                                debug!("Sending ping to websocket");
                                if let Err(err) = websocket_stream.send(tungstenite::Message::Ping(vec![])).await {
                                    error!("Error while sending ping to websocket: {}", err);
                                    daemon_status.write().websocket_status = Err(err.into());
                                    break;
                                };
                            }
                        }
                    }
                }
                Err(err) => {
                    error!("Error while connecting to websocket: {}", err);
                    daemon_status.write().websocket_status = Err(err);
                }
            }
            backoff.sleep().await;
        }
    });

    // Spawn settings watcher
    let daemon_status_clone = daemon_status.clone();
    let settings_watcher_join = tokio::spawn(async move {
        let daemon_status = daemon_status_clone;
        let mut backoff =
            Backoff::new(Duration::from_secs_f64(0.25), Duration::from_secs_f64(120.));
        loop {
            match spawn_settings_watcher(daemon_status.clone()).await {
                Ok(join_handle) => {
                    daemon_status.write().settings_watcher_status = Ok(());
                    backoff.reset();
                    if let Err(err) = join_handle.await {
                        error!("Error on settings watcher join: {:?}", err);
                        daemon_status.write().settings_watcher_status = Err(err.into());
                    }
                    return;
                }
                Err(err) => {
                    error!("Error spawning settings watcher: {:?}", err);
                    daemon_status.write().settings_watcher_status = Err(err);
                }
            }
            backoff.sleep().await;
        }
    });

    info!("Daemon is now running");

    match tokio::try_join!(
        scheduler_join,
        unix_join,
        websocket_join,
        settings_watcher_join,
    ) {
        Ok(_) => Ok(()),
        Err(err) => Err(err.into()),
    }
}
