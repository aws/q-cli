//! Utiities for IPC with Mac App

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use crate::proto::{local, FigProtobufEncodable};

use anyhow::Result;
use bytes::{Bytes, BytesMut};
use flume::{bounded, Receiver, Sender};
use log::{debug, error};
use prost::Message;
use tokio::{
    fs::remove_file,
    io::{AsyncReadExt, AsyncWriteExt},
    net::{UnixListener, UnixStream},
};

use crate::proto::figterm::{figterm_message, FigtermMessage, InsertTextCommand};

/// Get path to "$TMPDIR/fig.socket"
pub fn get_socket_path() -> PathBuf {
    [std::env::temp_dir().as_path(), Path::new("fig.socket")]
        .into_iter()
        .collect()
}

/// Connect to `socket` with a timeout
pub async fn connect_timeout(socket: impl AsRef<Path>, timeout: Duration) -> Result<UnixStream> {
    Ok(tokio::time::timeout(timeout, UnixStream::connect(socket)).await??)
}

/// Send a hook using a Unix socket
pub async fn send_hook(connection: &mut UnixStream, hook: local::hook::Hook) -> Result<()> {
    let message = local::LocalMessage {
        r#type: Some(local::local_message::Type::Hook(local::Hook {
            hook: Some(hook),
        })),
    };

    let encoded_message = message.encode_fig_protobuf()?;

    connection.write_all(&encoded_message).await?;
    Ok(())
}

pub async fn create_socket_listen(session_id: impl AsRef<str>) -> Result<UnixListener> {
    let session_id_str = session_id.as_ref().split(':').last().unwrap();

    let socket_path: PathBuf = [
        Path::new("/tmp"),
        Path::new(&format!("figterm-{}.socket", session_id_str)),
    ]
    .into_iter()
    .collect();

    // Remove the socket so we can create a new one
    if socket_path.exists() {
        remove_file(&socket_path).await?
    }

    Ok(UnixListener::bind(&socket_path)?)
}

pub async fn spawn_outgoing_sender() -> Result<Sender<Bytes>> {
    let (outgoing_tx, outgoing_rx) = bounded::<Bytes>(256);

    tokio::spawn(async move {
        let socket = get_socket_path();

        while let Ok(message) = outgoing_rx.recv_async().await {
            debug!(
                "Sending {} byte message to {}",
                message.len(),
                socket.display()
            );
            let conn = connect_timeout(&socket, Duration::from_secs(10)).await;

            // When on macOS after the socket connection is made a breif delay is required
            // Not sure why, but this is a workaround
            #[cfg(target_os = "macos")]
            tokio::time::sleep(Duration::from_millis(2)).await;

            match conn {
                Ok(mut unix_stream) => match unix_stream.write_all(&message).await {
                    Ok(_) => {
                        if let Err(e) = unix_stream.flush().await {
                            error!("Failed to flush socket: {}", e)
                        }
                    }
                    Err(e) => {
                        error!("Failed to send message: {}", e);
                    }
                },
                Err(e) => {
                    error!("Error connecting to socket: {}", e);
                }
            }
        }
    });

    Ok(outgoing_tx)
}

pub async fn spawn_incoming_receiver(
    session_id: impl AsRef<str>,
) -> Result<Receiver<FigtermMessage>> {
    let socket_listener = create_socket_listen(session_id).await?;
    let (incomming_tx, incomming_rx) = bounded(256);
    tokio::spawn(async move {
        loop {
            if let Ok((mut stream, _)) = socket_listener.accept().await {
                let incomming_tx = incomming_tx.clone();
                tokio::spawn(async move {
                    let mut buff = BytesMut::new();

                    loop {
                        match stream.read_buf(&mut buff).await {
                            Ok(0) => {
                                debug!("EOF from socket");
                                break;
                            }
                            Ok(n) => {
                                debug!("Read {} bytes from socket", n);
                            }
                            Err(e) => {
                                error!("Error reading from socket: {}", e);
                                return;
                            }
                        }
                    }

                    match FigtermMessage::decode(buff.as_ref()) {
                        Ok(message) => {
                            incomming_tx.clone().send_async(message).await.unwrap();
                        }
                        Err(e) => {
                            error!("Error decoding Figterm message: {}", e);
                            let text = String::from_utf8_lossy(buff.as_ref()).to_string();
                            let message = FigtermMessage {
                                command: Some(figterm_message::Command::InsertTextCommand(
                                    InsertTextCommand {
                                        insertion: Some(text),
                                        deletion: None,
                                        offset: None,
                                        immediate: None,
                                    },
                                )),
                            };
                            incomming_tx.clone().send_async(message).await.unwrap();
                        }
                    }
                });
            }
        }
    });

    Ok(incomming_rx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn socket_path_test() {
        assert!(get_socket_path().ends_with("fig.socket"))
    }
}
