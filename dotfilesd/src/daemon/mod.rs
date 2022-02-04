pub mod launchd_plist;
pub mod systemd_unit;
pub mod websocket;

use std::{
    io::Write,
    ops::ControlFlow,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use anyhow::{anyhow, Context, Result};
use futures::StreamExt;

use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use tokio::{fs::remove_file, net::UnixListener, select};

use crate::daemon::websocket::process_websocket;

use self::{launchd_plist::LaunchdPlist, systemd_unit::SystemdUnit};

fn daemon_log(message: &str) {
    println!(
        "[dotfiles-daemon {}] {}",
        time::OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .unwrap_or_else(|_| "xxxx-xx-xxTxx:xx:xx.xxxxxxZ".into()),
        message
    );
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitSystem {
    /// macOS init system
    ///
    /// https://launchd.info/
    Launchd,
    /// Most common Linux init system
    ///
    /// https://systemd.io/
    Systemd,
    /// Init system used by artix, void, etc
    ///
    /// http://smarden.org/runit/
    Runit,
    /// Init subsystem used by alpine, gentoo, etc
    ///
    /// https://wiki.gentoo.org/wiki/Project:OpenRC
    OpenRc,
}

impl InitSystem {
    pub fn get_init_system() -> Result<InitSystem> {
        let output = Command::new("ps 1").output().context("Could not run ps")?;

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

    pub fn daemon_status(&self, name: impl AsRef<str>) -> Result<Option<i32>> {
        match self {
            InitSystem::Launchd => {
                let output = Command::new("launchctl").arg("list").output()?;

                let stdout = String::from_utf8_lossy(&output.stdout);

                let status = stdout
                    .lines()
                    .map(|line| line.split_whitespace().collect::<Vec<_>>())
                    .find(|line| line[2] == name.as_ref())
                    .map(|data| data[1].parse::<i32>().ok())
                    .flatten();

                Ok(status)
            }
            InitSystem::Systemd => todo!(),
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

        let plist = LaunchdPlist::new("io.fig.dotfiles-daemon")
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebsocketAwsToken {
    access_token: String,
    id_token: String,
}

/// Spawn the daemon to listen for updates and dotfiles changes
pub async fn daemon() -> Result<()> {
    daemon_log("Starting daemon...");

    let mut update_interval = tokio::time::interval(Duration::from_secs(60 * 60));

    // Connect to websocket
    let mut websocket_stream = websocket::connect_to_fig_websocket()
        .await
        .context("Could not connect to websocket")?;

    // Connect to unix socket
    let unix_socket_path = Path::new("/tmp/dotfiles-daemon.sock");

    if unix_socket_path.exists() {
        remove_file(unix_socket_path).await?;
    }

    let unix_socket =
        UnixListener::bind(unix_socket_path).context("Could not connect to unix socket")?;

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
            _ = unix_socket.accept() => {

            }
            _ = update_interval.tick() => {
                // // Check for updates
                // match update(UpdateType::NoProgress)? {
                //     UpdateStatus::UpToDate => {}
                //     UpdateStatus::Updated(release) => {
                //         println!("Updated to {}", release.version);
                //         println!("Quitting...");
                //         return Ok(());
                //     }
                // }
            }
        }
    }

    Ok(())
}
