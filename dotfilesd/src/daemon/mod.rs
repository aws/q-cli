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
                Command::new("launchctl")
                    .arg("load")
                    .arg(path.as_ref())
                    .output()
                    .with_context(|| format!("Could not load {:?}", path.as_ref()))?;
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
}

pub struct DaemonService {
    pub path: PathBuf,
    pub data: &'static str,
    pub launch_system: LaunchSystem,
}

impl DaemonService {
    pub fn launchd() -> Option<DaemonService> {
        let basedirs = directories::BaseDirs::new()?;

        let path = basedirs
            .home_dir()
            .join("Library/LaunchAgents/io.fig.dotfiles-daemon.plist");

        let data = include_str!("daemon_files/io.fig.dotfiles-daemon.plist");

        Some(DaemonService {
            path,
            data,
            launch_system: LaunchSystem::Launchd,
        })
    }

    pub fn systemd() -> Option<DaemonService> {
        let basedirs = directories::BaseDirs::new()?;

        let path = basedirs
            .home_dir()
            .join(".config/systemd/user/fig-dotfiles-daemon.service");

        let data = include_str!("daemon_files/dotfiles-daemon.service");

        Some(DaemonService {
            path,
            data,
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
        self.launch_system.start_daemon(self.path.as_path())
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
