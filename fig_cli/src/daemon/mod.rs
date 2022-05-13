pub mod launchd_plist;
pub mod scheduler;
pub mod settings_watcher;
pub mod system_handler;
pub mod systemd_unit;
pub mod websocket;

use std::path::{
    Path,
    PathBuf,
};
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{
    anyhow,
    Context,
    Result,
};
use cfg_if::cfg_if;
use futures::{
    SinkExt,
    StreamExt,
};
use parking_lot::lock_api::RawMutex;
use parking_lot::{
    Mutex,
    RwLock,
};
use rand::distributions::Uniform;
use rand::prelude::Distribution;
use tokio::select;
use tokio_tungstenite::tungstenite;
use tracing::{
    debug,
    error,
    info,
};

use crate::daemon::launchd_plist::LaunchdPlist;
use crate::daemon::settings_watcher::spawn_settings_watcher;
use crate::daemon::systemd_unit::SystemdUnit;
use crate::daemon::websocket::process_websocket;
use crate::util::backoff::Backoff;

pub fn get_daemon() -> Result<LaunchService> {
    cfg_if! {
        if #[cfg(target_os = "macos")] {
            LaunchService::launchd()
        } else if #[cfg(target_os = "linux")] {
            LaunchService::systemd()
        } else if #[cfg(windows)] {
            LaunchService::scm()
        } else {
            Err(anyhow!("Unsupported platform"));
        }
    }
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
    /// Init subsystem used by Windows
    ///
    /// <https://docs.microsoft.com/en-us/windows/win32/services/service-control-manager>
    SCM,
}

impl InitSystem {
    pub fn get_init_system() -> Result<InitSystem> {
        let output = Command::new("ps").args(["-p1"]).output().context("Could not run ps")?;

        if !output.status.success() {
            return Err(anyhow!("ps failed: {}", output.status));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);

        if output_str.contains("launchd") {
            Ok(InitSystem::Launchd)
        } else if output_str.contains("systemd") {
            Ok(InitSystem::Systemd)
        } else if output_str.contains("runit") {
            Ok(InitSystem::Runit)
        } else if output_str.contains("openrc") {
            Ok(InitSystem::OpenRc)
        } else if output_str.contains("sc") {
            Ok(InitSystem::SCM)
        } else {
            Err(anyhow!("Could not determine init system"))
        }
    }

    pub fn start_daemon(&self, path: impl AsRef<Path>) -> Result<()> {
        match self {
            InitSystem::Launchd => {
                let output = Command::new("launchctl").arg("load").arg(path.as_ref()).output()?;

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
            },
            InitSystem::Systemd => {
                let output = Command::new("systemctl")
                    .arg("--user")
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
            },
            InitSystem::SCM => {
                let output = Command::new("sc")
                    .arg("start")
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
            },
            _ => Err(anyhow!("Could not start daemon: unsupported init system")),
        }
    }

    fn stop_daemon(&self, path: impl AsRef<Path>) -> Result<()> {
        match self {
            InitSystem::Launchd => {
                let output = Command::new("launchctl").arg("unload").arg(path.as_ref()).output()?;

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
            },
            InitSystem::Systemd => {
                Command::new("systemctl")
                    .arg("--now")
                    .arg("disable")
                    .arg(path.as_ref())
                    .output()
                    .with_context(|| format!("Could not disable {:?}", path.as_ref()))?;

                Ok(())
            },
            InitSystem::SCM => {
                Command::new("sc")
                    .arg("stop")
                    .arg(path.as_ref())
                    .output()
                    .with_context(|| format!("Could not disable {:?}", path.as_ref()))?;

                Ok(())
            },
            _ => Err(anyhow!("Could not stop daemon: unsupported init system")),
        }
    }

    pub fn restart_daemon(&self, path: impl AsRef<Path>) -> Result<()> {
        self.stop_daemon(path.as_ref()).ok();
        self.start_daemon(path.as_ref())?;

        // TODO: use restart functionality of init system if possible
        // note that windows doesn't appear to have this functionality

        Ok(())
    }

    pub fn daemon_name(&self) -> &'static str {
        match self {
            InitSystem::Launchd => "io.fig.dotfiles-daemon",
            InitSystem::Systemd | InitSystem::SCM => "fig-daemon",
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
                    .find(|line| line.get(2) == Some(&self.daemon_name()))
                    .and_then(|data| data.get(1).and_then(|v| v.parse::<i32>().ok()));

                Ok(status)
            },
            InitSystem::Systemd => {
                let output = Command::new("systemctl")
                    .arg("--user")
                    .arg("show")
                    .arg("-pExecMainStatus")
                    .arg(format!("{}.service", self.daemon_name()))
                    .output()?;

                let stdout = String::from_utf8_lossy(&output.stdout);

                let status = stdout.split('=').last().and_then(|s| s.trim().parse::<i32>().ok());

                Ok(status)
            },
            InitSystem::SCM => {
                let _output = Command::new("sc")
                    .arg("query")
                    .arg("type=")
                    .arg("service")
                    .output()
                    .context("Could not query SCM")?;

                todo!("Parse service status and return it (windows)");
            },
            _ => Err(anyhow!("Could not get daemon status: unsupported init system")),
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

        let log_path = homedir.join(".fig").join("logs").join("daemon.log");
        let log_path_str = format!("file:{}", log_path.to_string_lossy());

        let unit = SystemdUnit::new("Fig Dotfiles Daemon")
            .exec_start(format!("{} daemon", executable_path_str))
            .restart("always")
            .restart_sec(5)
            .wanted_by("default.target")
            .standard_output(&*log_path_str)
            .standard_error(&*log_path_str)
            .unit();

        Ok(LaunchService {
            path,
            data: unit,
            launch_system: InitSystem::Systemd,
        })
    }

    pub fn scm() -> Result<LaunchService> {
        let _homedir = fig_directories::home_dir().context("Could not get home directory")?;

        todo!("Figure out windows SCM launch call");
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
        // Create parent directory
        std::fs::create_dir_all(&self.path.parent().context("Could not get parent directory")?)?;

        // Write to the definition file
        std::fs::write(&self.path, self.data.as_bytes())?;

        // Restart the daemon
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
    system_socket_status: Result<()>,
}

impl Default for DaemonStatus {
    fn default() -> Self {
        Self {
            time_started: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("System time set before unix epoch")
                .as_secs(),
            settings_watcher_status: Ok(()),
            websocket_status: Ok(()),
            system_socket_status: Ok(()),
        }
    }
}

pub static IS_RUNNING_DAEMON: Mutex<bool> = Mutex::const_new(RawMutex::INIT, false);

/// Spawn the daemon to listen for updates and dotfiles changes
#[cfg(unix)]
pub async fn daemon() -> Result<()> {
    use crate::daemon::system_handler::spawn_incoming_system_handler;

    *IS_RUNNING_DAEMON.lock() = true;

    info!("Starting daemon...");

    let daemon_status = Arc::new(RwLock::new(DaemonStatus::default()));

    // Add small random element to the delay to avoid all clients from sending the messages at the same
    // time
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
        let mut backoff = Backoff::new(Duration::from_secs_f64(0.25), Duration::from_secs_f64(120.));
        loop {
            match spawn_incoming_system_handler(daemon_status.clone()).await {
                Ok(handle) => {
                    daemon_status.write().system_socket_status = Ok(());
                    backoff.reset();
                    if let Err(err) = handle.await {
                        error!("Error on system handler join: {:?}", err);
                        daemon_status.write().system_socket_status = Err(err.into());
                    }
                    return;
                },
                Err(err) => {
                    error!("Error spawning system handler: {:?}", err);
                    daemon_status.write().system_socket_status = Err(err);
                },
            }
            backoff.sleep().await;
        }
    });

    // Spawn websocket handler
    let daemon_status_clone = daemon_status.clone();
    let websocket_join = tokio::spawn(async move {
        let daemon_status = daemon_status_clone;
        let mut backoff = Backoff::new(Duration::from_secs_f64(0.25), Duration::from_secs_f64(120.));
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
                },
                Err(err) => {
                    error!("Error while connecting to websocket: {}", err);
                    daemon_status.write().websocket_status = Err(err);
                },
            }
            backoff.sleep().await;
        }
    });

    // Spawn settings watcher
    let daemon_status_clone = daemon_status.clone();
    let settings_watcher_join = tokio::spawn(async move {
        let daemon_status = daemon_status_clone;
        let mut backoff = Backoff::new(Duration::from_secs_f64(0.25), Duration::from_secs_f64(120.));
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
                },
                Err(err) => {
                    error!("Error spawning settings watcher: {:?}", err);
                    daemon_status.write().settings_watcher_status = Err(err);
                },
            }
            backoff.sleep().await;
        }
    });

    info!("Daemon is now running");

    match tokio::try_join!(scheduler_join, unix_join, websocket_join, settings_watcher_join,) {
        Ok(_) => Ok(()),
        Err(err) => Err(err.into()),
    }
}

/// Spawn the daemon to listen for updates and dotfiles changes
#[cfg(windows)]
pub async fn daemon() -> Result<()> {
    todo!();
}
