pub mod launchd_plist;
pub mod systemd_unit;
pub mod websocket;

use crate::{
    daemon::{
        launchd_plist::LaunchdPlist, systemd_unit::SystemdUnit, websocket::process_websocket,
    },
    util::settings::Settings,
};

use anyhow::{anyhow, Context, Result};
use fig_ipc::{
    daemon::get_daemon_socket_path, hook::send_settings_changed, recv_message, send_message,
};
use fig_proto::daemon::diagnostic_response::{SettingsWatcherStatus, WebsocketStatus};
use futures::StreamExt;
use notify::{watcher, RecursiveMode, Watcher};
use parking_lot::RwLock;
use std::{
    io::Write,
    ops::ControlFlow,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
    time::Duration,
};
use time::format_description::well_known::Rfc3339;
use tokio::{
    fs::remove_file,
    net::{UnixListener, UnixStream},
    select,
};

fn daemon_log(message: &str) {
    println!(
        "[dotfiles-daemon {}] {}",
        time::OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .unwrap_or_else(|_| "xxxx-xx-xxTxx:xx:xx.xxxxxxZ".into()),
        message
    );
}

pub fn install_daemon() -> Result<()> {
    #[cfg(target_os = "macos")]
    LaunchService::launchd()?.install()?;
    #[cfg(target_os = "linux")]
    LaunchService::systemd()?.install()?;
    #[cfg(target_os = "windows")]
    return Err(anyhow::anyhow!("Windows is not yet supported"));
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    return Err(anyhow::anyhow!("Unsupported platform"));

    Ok(())
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

    fn start_daemon(&self, path: impl AsRef<Path>) -> Result<()> {
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
                Command::new("systemctl")
                    .arg("--now")
                    .arg("enable")
                    .arg(path.as_ref())
                    .output()
                    .with_context(|| format!("Could not enable {:?}", path.as_ref()))?;

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
                    .map(|data| data[1].parse::<i32>().ok())
                    .flatten();

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
        let basedirs = directories::BaseDirs::new().context("Could not get base directories")?;

        let plist_path = basedirs
            .home_dir()
            .join("Library/LaunchAgents/io.fig.dotfiles-daemon.plist");

        let executable_path = std::env::current_exe()?;
        let executable_path_str = executable_path.to_string_lossy();

        let log_path = basedirs.home_dir().join(".fig/logs/dotfiles-daemon.log");
        let log_path_str = log_path.to_string_lossy();

        let plist = LaunchdPlist::new(InitSystem::Launchd.daemon_name())
            .program(&*executable_path_str)
            .program_arguments([&*executable_path_str, "daemon"])
            .keep_alive(true)
            .run_at_load(true)
            .throttle_interval(5)
            .standard_out_path(&*log_path_str)
            .standard_error_path(&*log_path_str)
            .plist();

        Ok(LaunchService {
            path: plist_path,
            data: plist,
            launch_system: InitSystem::Launchd,
        })
    }

    pub fn systemd() -> Result<LaunchService> {
        let basedirs = directories::BaseDirs::new().context("Could not get base directories")?;

        let path = basedirs
            .home_dir()
            .join(".config/systemd/user/fig-dotfiles-daemon.service");

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

    pub fn install(&self) -> Result<()> {
        // Write to the definition file
        let mut file = std::fs::File::create(&self.path)?;
        file.write_all(self.data.as_bytes())?;

        // Restart the daemon
        self.launch_system.restart_daemon(self.path.as_path())?;

        Ok(())
    }

    pub fn uninstall(&self) -> Result<()> {
        // Stop the daemon
        self.launch_system.stop_daemon(self.path.as_path()).ok();

        // Remove the definition file
        std::fs::remove_file(&self.path)?;

        Ok(())
    }
}

struct DaemonStatus {
    /// The time the daemon was started as a u64 timestamp in seconds since the epoch
    time_started: u64,
    settings_watcher_status: Result<()>,
}

async fn spawn_settings_watcher() -> Result<flume::Receiver<Result<()>>> {
    let settings_path = Settings::path()?;

    let (settings_watcher_tx, settings_watcher_rx) = std::sync::mpsc::channel();
    let mut watcher = watcher(settings_watcher_tx, Duration::from_secs(1))?;

    let (forward_tx, forward_rx) = flume::unbounded();
    let (status_tx, status_rx) = flume::unbounded();

    tokio::task::spawn(async move {
        loop {
            match forward_rx.recv_async().await {
                Ok(_) => match send_settings_changed().await {
                    Ok(_) => daemon_log("Settings changed"),
                    Err(err) => {
                        daemon_log(&format!("Could not send settings changed: {}", err));
                        status_tx.send_async(Err(err)).await.ok();
                    }
                },
                Err(err) => {
                    daemon_log(&format!("Error while receiving settings: {}", err));
                    status_tx.send_async(Err(err.into())).await.ok();
                }
            }
        }
    });

    std::thread::spawn(
        move || match watcher.watch(&settings_path, RecursiveMode::NonRecursive) {
            Ok(()) => loop {
                match settings_watcher_rx.recv() {
                    Ok(event) => {
                        if let Err(e) = forward_tx.send(event) {
                            daemon_log(&format!("Error forwarding settings event: {}", e));
                        }
                    }
                    Err(err) => {
                        eprintln!("{}", err);
                    }
                }
            },
            Err(err) => {
                daemon_log(&format!("Error while watching settings: {}", err));
            }
        },
    );

    Ok(status_rx)
}

async fn spawn_unix_handler(
    mut stream: UnixStream,
    daemon_status: Arc<RwLock<DaemonStatus>>,
) -> Result<()> {
    tokio::task::spawn(async move {
        loop {
            match recv_message::<fig_proto::daemon::DaemonMessage, _>(&mut stream).await {
                Ok(msg) => {
                    println!("Received message: {:?}", msg);

                    let message = {
                        let daemon_status = daemon_status.read();

                        let settings_status = match daemon_status.settings_watcher_status {
                            Ok(_) => SettingsWatcherStatus::SettingsWatcherOk,
                            Err(_) => SettingsWatcherStatus::SettingsWatcherError,
                        };

                        fig_proto::daemon::new_diagnostic_response(
                            daemon_status.time_started,
                            settings_status,
                            WebsocketStatus::WebsocketOk,
                        )
                    };

                    if let Err(err) = send_message(&mut stream, message).await {
                        eprintln!("Error sending message: {}", err);
                    }
                }
                Err(err) => {
                    eprintln!("Error while receiving message: {}", err);
                }
            }
        }
    });

    Ok(())
}

/// Spawn the daemon to listen for updates and dotfiles changes
pub async fn daemon() -> Result<()> {
    daemon_log("Starting daemon...");

    let daemon_status = Arc::new(RwLock::new(DaemonStatus {
        time_started: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
        settings_watcher_status: Ok(()),
    }));

    let mut update_interval = tokio::time::interval(Duration::from_secs(60 * 60));

    // Connect to websocket
    let mut websocket_stream = websocket::connect_to_fig_websocket()
        .await
        .context("Could not connect to websocket")?;

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

    let spawn_settings_watcher_result = spawn_settings_watcher().await;
    if let Err(err) = &spawn_settings_watcher_result {
        daemon_log(&format!("Could not spawn settings watcher: {}", err));
    }

    daemon_log("Daemon is now running");

    // Select loop
    loop {
        select! {
            next = websocket_stream.next() => {
                match process_websocket(&next).await? {
                    ControlFlow::Continue(_) => {},
                    ControlFlow::Break(_) => break,
                }
            }
            conn = unix_socket.accept() => {
                match conn {
                    Ok((stream, _)) => {
                        spawn_unix_handler(stream, daemon_status.clone()).await?;
                    }
                    Err(err) => {
                        daemon_log(&format!("Could not accept unix socket connection: {}", err));
                    }
                }
            }

            _ = update_interval.tick() => {
                #[cfg(feature = "auto-update")]
                {
                    // Check for updates
                    match update(UpdateType::NoProgress)? {
                        UpdateStatus::UpToDate => {}
                        UpdateStatus::Updated(release) => {
                            println!("Updated to {}", release.version);
                            println!("Quitting...");
                            return Ok(());
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
