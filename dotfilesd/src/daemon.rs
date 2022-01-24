use std::{path::Path, time::Duration};

use anyhow::Result;
use futures_util::StreamExt;
use self_update::update::UpdateStatus;
use tokio::{fs::remove_file, io::AsyncReadExt, net::UnixStream, select};
use tokio_tungstenite::tungstenite::Message;

use crate::cli::{sync, update, UpdateType};

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
                                        sync().await?;
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
