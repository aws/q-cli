//! Utiities for IPC with Mac App

use anyhow::Result;
use fig_proto::figterm::FigtermMessage;
use flume::{unbounded, Receiver, Sender};
use std::time::Duration;
use tokio::{fs::remove_file, io::AsyncWriteExt, net::UnixListener};
use tracing::{debug, error, trace};

pub async fn create_socket_listen(session_id: impl AsRef<str>) -> Result<UnixListener> {
    let socket_path = fig_ipc::figterm::get_figterm_socket_path(session_id);

    // Remove the socket so we can create a new one
    if socket_path.exists() {
        remove_file(&socket_path).await?
    }

    Ok(UnixListener::bind(&socket_path)?)
}

pub async fn remove_socket(session_id: impl AsRef<str>) -> Result<()> {
    let socket_path = fig_ipc::figterm::get_figterm_socket_path(session_id);

    if socket_path.exists() {
        remove_file(&socket_path).await?
    }

    Ok(())
}

pub async fn spawn_outgoing_sender() -> Result<Sender<fig_proto::local::LocalMessage>> {
    trace!("Spawning outgoing sender");
    let (outgoing_tx, outgoing_rx) = unbounded::<fig_proto::local::LocalMessage>();

    tokio::spawn(async move {
        let socket = fig_ipc::get_fig_socket_path();

        while let Ok(message) = outgoing_rx.recv_async().await {
            match fig_ipc::connect_timeout(&socket, Duration::from_secs(1)).await {
                Ok(mut unix_stream) => {
                    match fig_ipc::send_message(&mut unix_stream, message).await {
                        Ok(()) => {
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
    trace!("Spawning incoming receiver");

    let socket_listener = create_socket_listen(session_id).await?;
    let (incomming_tx, incomming_rx) = unbounded();

    tokio::spawn(async move {
        loop {
            if let Ok((mut stream, addr)) = socket_listener.accept().await {
                trace!("Accepted connection from {:?}", addr);
                let incomming_tx = incomming_tx.clone();
                tokio::spawn(async move {
                    loop {
                        match fig_ipc::recv_message::<FigtermMessage, _>(&mut stream).await {
                            Ok(Some(message)) => {
                                debug!("Received message: {:?}", message);
                                incomming_tx.clone().send_async(message).await.unwrap();
                            }
                            Ok(None) => {
                                debug!("Received EOF");
                                break;
                            }
                            Err(err) => {
                                error!("Error receiving message: {}", err);
                                break;
                            }
                        }
                    }
                });
            }
        }
    });

    Ok(incomming_rx)
}
