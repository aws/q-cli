use std::{path::Path, time::Duration, io::Write};

use anyhow::Result;
use futures_util::StreamExt;
use self_update::update::UpdateStatus;
use tokio::{fs::remove_file, io::AsyncReadExt, net::UnixStream, select};
use tokio_tungstenite::tungstenite::Message;

use crate::cli::{
    installation::{update, UpdateType},
    sync,
};

pub struct DaemonService {
    pub path: &'static Path,
    pub data: &'static str,
}

impl DaemonService {
    pub fn write_to_file(&self) -> Result<()> {
        let mut file = std::fs::File::create(self.path)?;
        file.write_all(self.data.as_bytes())?;
        Ok(())
    }
}

#[cfg(target_os = "linux")]
pub fn systemd_service() -> DaemonService {
    let path = Path::new("/etc/systemd/system/dotfiles-daemon.service");
    let data = include_str!("daemon_files/dotfiles-daemon.service");

    DaemonService { path, data }
}

#[cfg(target_os = "macos")]
pub fn launchd_plist() -> DaemonService {
    let path = Path::new("/Library/LaunchDaemons/io.fig.dotfiles-daemon.plist");
    let data = include_str!("daemon_files/io.fig.dotfiles-daemon.plist");

    DaemonService { path, data }
}

pub async fn daemon() -> Result<()> {
    // Spawn the daemon to listen for updates and dotfiles changes
    let mut update_interval = tokio::time::interval(Duration::from_secs(60 * 60));

    // Connect to websocket
    let (websocket_stream, _) = tokio_tungstenite::connect_async("ws://127.0.0.1:1234").await?;

    let (_write, mut read) = websocket_stream.split();

    let unix_socket_path = Path::new("/var/run/dotfiles-daemon.sock");

    if unix_socket_path.exists() {
        remove_file(unix_socket_path).await?;
    }

    let mut unix_socket = UnixStream::connect("/var/run/dotfiles-daemon.sock").await?;

    let mut bytes = bytes::BytesMut::new();

    loop {
        select! {
            next = read.next() => {
                match next {
                    Some(stream_result) => match stream_result {
                        Ok(message) => match message {
                            Message::Text(text) => {
                                match text.trim() {
                                    "dotfiles" => {
                                        sync::sync_all_files().await?;
                                    }
                                    text => {
                                        println!("Received unknown text: {}", text);
                                    }
                                }
                            }
                            message => {
                                println!("Received unknown message: {:?}", message);
                            }
                        },
                        Err(err) => {
                            // TODO: Gracefully handle errors
                            println!("Error: {:?}", err);
                            continue;
                        }
                    },
                    None => {
                        // TODO: Handle disconnections
                        return Err(anyhow::anyhow!("Websocket disconnected"));
                    }
                }
            }
            _ = unix_socket.read_buf(&mut bytes) => {

            }
            _ = update_interval.tick() => {
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
