//! Utiities for IPC with Mac App

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use crate::proto::local;

use anyhow::Result;
use bytes::{Bytes, BytesMut};
use log::{error, trace};
use prost::Message;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{UnixListener, UnixStream},
    sync::mpsc::{Receiver, UnboundedSender},
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

    let encoded_message = message.to_fig_pbuf()?;

    connection.write_all(&encoded_message).await?;
    Ok(())
}

pub async fn create_socket_listen(session_id: impl AsRef<str>) -> Result<UnixListener> {
    let s = session_id.as_ref().split(':').last().unwrap();

    let path: PathBuf = [
        Path::new("/tmp"),
        Path::new(&format!("figterm-{}.socket", s)),
    ]
    .into_iter()
    .collect();

    // Remove the socket
    match tokio::fs::remove_file(&path).await {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
        _ => Ok(()),
    }?;

    Ok(UnixListener::bind(&path)?)
}

pub async fn spawn_outgoing_sender() -> Result<UnboundedSender<Bytes>> {
    let (outgoing_tx, mut outgoing_rx) = tokio::sync::mpsc::unbounded_channel::<Bytes>();

    tokio::spawn(async move {
        let socket = get_socket_path();

        while let Some(message) = outgoing_rx.recv().await {
            match connect_timeout(socket.clone(), Duration::from_secs(1)).await {
                Ok(mut unix_stream) => {
                    unix_stream.write_all(&message).await.unwrap();
                    unix_stream.flush().await.unwrap();
                }
                Err(e) => {
                    error!("Error sending message: {}", e);
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
    let (incomming_tx, incomming_rx) = tokio::sync::mpsc::channel(128);
    tokio::spawn(async move {
        loop {
            if let Ok((mut stream, _)) = socket_listener.accept().await {
                let incomming_tx = incomming_tx.clone();
                tokio::spawn(async move {
                    let mut buff = BytesMut::new();

                    loop {
                        match stream.read_buf(&mut buff).await {
                            Ok(0) => {
                                trace!("EOF from socket");
                                break;
                            }
                            Ok(n) => {
                                trace!("Read {} bytes from socket", n);
                            }
                            Err(e) => {
                                error!("Error reading from socket: {}", e);
                                return;
                            }
                        }
                    }

                    match FigtermMessage::decode(buff.as_ref()) {
                        Ok(message) => {
                            incomming_tx.clone().send(message).await.unwrap();
                        }
                        Err(e) => {
                            error!("Error decoding Figterm message: {}", e);
                            let message = FigtermMessage {
                                command: Some(figterm_message::Command::InsertTextCommand(
                                    InsertTextCommand {
                                        text: String::from_utf8_lossy(buff.as_ref()).into(),
                                        clear: false,
                                    },
                                )),
                            };
                            incomming_tx.clone().send(message).await.unwrap();
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
