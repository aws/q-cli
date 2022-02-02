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

use anyhow::{Context, Result};
use futures::StreamExt;

use serde::{Deserialize, Serialize};
use tokio::{fs::remove_file, net::UnixListener, select};

use crate::daemon::websocket::process_websocket;

use self::{launchd_plist::LaunchdPlist, systemd_unit::SystemdUnit};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitSystem {
    Systemd,
}

#[cfg(target_os = "linux")]
pub fn get_init_system() -> Result<InitSystem> {
    use std::process::Command;

    use anyhow::Context;

    let output = Command::new("ps 1")
        .output()
        .with_context(|| "Could not get init system")?;

    let stdout = String::from_utf8(output.stdout).with_context(|| "Could not parse init system")?;

    if stdout.contains("systemd") {
        Ok(InitSystem::Systemd)
    } else {
        Err(anyhow::anyhow!("Could not determine init system"))
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchSystem {
    Launchd,
    Systemd,
}

impl LaunchSystem {
    fn start_daemon(&self, path: impl AsRef<Path>) -> Result<()> {
        match self {
            LaunchSystem::Launchd => {
                let output = Command::new("launchctl")
                    .arg("load")
                    .arg(path.as_ref())
                    .output()?;

                if !output.status.success() {
                    return Err(anyhow::anyhow!(
                        "Could not start daemon: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }

                let stderr = String::from_utf8_lossy(&output.stderr);

                if !stderr.is_empty() {
                    return Err(anyhow::anyhow!(
                        "Could not start daemon: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }
            }
            LaunchSystem::Systemd => {
                Command::new("systemctl")
                    .arg("--now")
                    .arg("enable")
                    .arg(path.as_ref())
                    .output()
                    .with_context(|| format!("Could not enable {:?}", path.as_ref()))?;
            }
        }
        Ok(())
    }

    fn stop_daemon(&self, path: impl AsRef<Path>) -> Result<()> {
        match self {
            LaunchSystem::Launchd => {
                let output = Command::new("launchctl")
                    .arg("unload")
                    .arg(path.as_ref())
                    .output()?;

                if !output.status.success() {
                    return Err(anyhow::anyhow!(
                        "Could not stop daemon: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }

                let stderr = String::from_utf8_lossy(&output.stderr);

                if !stderr.is_empty() {
                    return Err(anyhow::anyhow!(
                        "Could not stop daemon: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }
            }
            LaunchSystem::Systemd => {
                Command::new("systemctl")
                    .arg("--now")
                    .arg("disable")
                    .arg(path.as_ref())
                    .output()
                    .with_context(|| format!("Could not disable {:?}", path.as_ref()))?;
            }
        }
        Ok(())
    }

    pub fn daemon_status(&self, name: impl AsRef<str>) -> Result<Option<i32>> {
        match self {
            LaunchSystem::Launchd => {
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
            LaunchSystem::Systemd => todo!(),
        }
    }
}

pub struct DaemonService {
    pub path: PathBuf,
    pub data: String,
    pub launch_system: LaunchSystem,
}

impl DaemonService {
    pub fn launchd() -> Option<DaemonService> {
        let basedirs = directories::BaseDirs::new()?;

        let path = basedirs
            .home_dir()
            .join("Library/LaunchAgents/io.fig.dotfiles-daemon.plist");

        let executable_path = std::env::current_exe().ok()?;
        let executable_path_str = executable_path.to_string_lossy().to_string();

        let plist = LaunchdPlist::new("io.fig.dotfiles-daemon")
            .program(&*executable_path_str)
            .program_arguments([&*executable_path_str, "daemon"])
            .keep_alive(true)
            .plist();

        Some(DaemonService {
            path,
            data: plist,
            launch_system: LaunchSystem::Launchd,
        })
    }

    pub fn systemd() -> Option<DaemonService> {
        let basedirs = directories::BaseDirs::new()?;

        let path = basedirs
            .home_dir()
            .join(".config/systemd/user/fig-dotfiles-daemon.service");

        let executable_path = std::env::current_exe().ok()?;
        let executable_path_str = executable_path.to_string_lossy();

        let unit = SystemdUnit::new("Fig Dotfiles Daemon")
            .exec_start(executable_path_str)
            .restart("always")
            .restart_sec(5)
            .wanted_by("default.target")
            .unit();

        Some(DaemonService {
            path,
            data: unit,
            launch_system: LaunchSystem::Systemd,
        })
    }

    pub fn write_to_file(&self) -> Result<()> {
        let mut file = std::fs::File::create(&self.path)?;
        file.write_all(self.data.as_bytes())?;
        Ok(())
    }

    pub fn install(&self) -> Result<()> {
        self.write_to_file()?;
        self.launch_system.stop_daemon(self.path.as_path()).ok();
        self.launch_system.start_daemon(self.path.as_path())?;
        Ok(())
    }
}

#[cfg(target_os = "macos")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebsocketAwsToken {
    access_token: String,
    id_token: String,
}

pub async fn daemon() -> Result<()> {
    // Spawn the daemon to listen for updates and dotfiles changes
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

    println!("Daemon is running...");

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
