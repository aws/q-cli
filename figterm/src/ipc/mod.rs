//! Utiities for IPC with Mac App

use anyhow::Result;
use fig_proto::figterm::FigtermMessage;
use flume::{bounded, Receiver, Sender};
use std::time::Duration;
use tokio::{fs::remove_file, io::AsyncWriteExt, net::UnixListener};
use tracing::{debug, error};

pub async fn create_socket_listen(session_id: impl AsRef<str>) -> Result<UnixListener> {
    let socket_path = fig_ipc::figterm::get_figterm_socket_path(session_id);

    // Remove the socket so we can create a new one
    if socket_path.exists() {
        remove_file(&socket_path).await?
    }

    Ok(UnixListener::bind(&socket_path)?)
}

pub async fn spawn_outgoing_sender() -> Result<Sender<fig_proto::local::LocalMessage>> {
    let (outgoing_tx, outgoing_rx) = bounded::<fig_proto::local::LocalMessage>(256);

    tokio::spawn(async move {
        let socket = fig_ipc::get_fig_socket_path();

        while let Ok(message) = outgoing_rx.recv_async().await {
            let conn = fig_ipc::connect_timeout(&socket, Duration::from_secs(1)).await;

            // When on macOS after the socket connection is made a breif delay is required
            // Not sure why, but this is a workaround
            #[cfg(target_os = "macos")]
            tokio::time::sleep(Duration::from_millis(2)).await;

            match conn {
                Ok(mut unix_stream) => {
                    match fig_ipc::send_message(&mut unix_stream, message).await {
                        Ok(_) => {
                            if let Err(e) = unix_stream.flush().await {
                                error!("Failed to flush socket: {}", e)
                            }
                        }
                        Err(e) => {
                            error!("Failed to send message: {}", e);
                        }
                    }
                }
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
                    loop {
                        match fig_ipc::recv_message::<FigtermMessage, _>(&mut stream).await {
                            Ok(message) => {
                                debug!("Received message: {:?}", message);
                                incomming_tx.clone().send_async(message).await.unwrap();
                            }
                            Err(err) => {
                                error!("Error receiving message: {}", err);
                            }
                        }
                    }
                });
            }
        }
    });

    Ok(incomming_rx)
}
