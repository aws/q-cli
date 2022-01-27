use std::{io::Write, path::Path, time::Duration};

use anyhow::Result;
use futures_util::StreamExt;
use self_update::update::UpdateStatus;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::remove_file,
    io::AsyncReadExt,
    net::{TcpStream, UnixStream},
    select,
};
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::{
    auth::Credentials,
    cli::{
        installation::{update, UpdateType},
        sync,
    },
};

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebsocketAwsToken {
    access_token: String,
    id_token: String,
}

pub async fn connect_to_fig_websocket() -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
    let client = reqwest::Client::new();

    let creds = Credentials::load_credentials()?;

    let websocket_aws_token = match (creds.access_token, creds.id_token) {
        (Some(access_token), Some(id_token)) => WebsocketAwsToken {
            access_token,
            id_token,
        },
        _ => {
            return Err(anyhow::anyhow!("Could not get AWS credentials"));
        }
    };

    let base64_token = base64::encode(&serde_json::to_string(&websocket_aws_token)?);

    let response = client
        .get("https://api.fig.io/authenticate/ticket")
        .bearer_auth(&base64_token)
        .send()
        .await?
        .text()
        .await?;

    println!("{:?}", response);

    let url = url::Url::parse_with_params(
        "wss://api.fig.io/",
        &[("deviceId", "1234"), ("ticket", &response)],
    )?;

    println!("{:?}", url);

    tokio::task::spawn(async move {
        // Wait for 5 seconds
        tokio::time::sleep(Duration::from_secs(2)).await;

        let json = r#"{"block": {"type": "alias","data": { "name": "HOMEBREW_NO_AUTO_UPDATE", "command": "" },"component": "abc","include": { "shell": [] }}}"#;

        let response = reqwest::Client::new()
            .post("https://api.fig.io/dotfiles/block/create")
            .body(json)
            .bearer_auth(&base64_token)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        println!("{:?}", response);
    });

    let (mut client, _) = tokio_tungstenite::connect_async(url).await?;

    while let Some(message) = client.next().await {
        println!("{:?}", message);
    }

    Ok(client)
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
