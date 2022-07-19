//! Utiities for IPC with Mac App

use std::time::Duration;

use anyhow::Result;
use fig_proto::figterm::{
    FigtermMessage,
    FigtermResponse,
};
use fig_proto::FigProtobufEncodable;
use flume::{
    unbounded,
    Receiver,
    Sender,
};
use tokio::fs::remove_file;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixListener;
use tracing::{
    debug,
    error,
    trace,
};

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
        remove_file(&socket_path).await?;
    }

    Ok(())
}

/// Spawn a thread to send events to Fig desktop app
pub async fn spawn_outgoing_sender() -> Result<Sender<fig_proto::local::LocalMessage>> {
    trace!("Spawning outgoing sender");
    let (outgoing_tx, outgoing_rx) = unbounded::<fig_proto::local::LocalMessage>();

    tokio::spawn(async move {
        let socket = fig_ipc::get_fig_socket_path();

        while let Ok(message) = outgoing_rx.recv_async().await {
            match fig_ipc::connect_timeout(&socket, Duration::from_secs(1)).await {
                Ok(mut unix_stream) => match fig_ipc::send_message(&mut unix_stream, message).await {
                    Ok(()) => {
                        if let Err(e) = unix_stream.flush().await {
                            error!("Failed to flush socket: {e}");
                        }
                    },
                    Err(e) => error!("Failed to send message: {e}"),
                },
                Err(e) => error!("Error connecting to socket: {e}"),
            }
        }
    });

    Ok(outgoing_tx)
}

pub async fn spawn_incoming_receiver(
    session_id: impl AsRef<str>,
) -> Result<Receiver<(FigtermMessage, Sender<FigtermResponse>)>> {
    trace!("Spawning incoming receiver");

    let socket_listener = create_socket_listen(session_id).await?;
    let (incoming_tx, incoming_rx) = unbounded();

    tokio::spawn(async move {
        loop {
            if let Ok((stream, addr)) = socket_listener.accept().await {
                trace!("Accepted connection from {addr:?}");

                let incoming_tx = incoming_tx.clone();

                let (mut read_half, mut write_half) = tokio::io::split(stream);
                let (response_tx, response_rx) = unbounded::<FigtermResponse>();

                tokio::spawn(async move {
                    let mut rx_thread = tokio::spawn(async move {
                        loop {
                            match fig_ipc::recv_message::<FigtermMessage, _>(&mut read_half).await {
                                Ok(Some(message)) => {
                                    debug!("Received message: {message:?}");
                                    incoming_tx
                                        .clone()
                                        .send_async((message, response_tx.clone()))
                                        .await
                                        .unwrap();
                                },
                                Ok(None) => {
                                    debug!("Received EOF");
                                    break;
                                },
                                Err(err) => {
                                    error!("Error receiving message: {err}");
                                    break;
                                },
                            }
                        }
                    });

                    loop {
                        tokio::select! {
                            // Break once the rx_thread quits
                            _ = &mut rx_thread => break,
                            res = response_rx.recv_async() => {
                                match res {
                                    Ok(message) => {
                                        match message.encode_fig_protobuf() {
                                            Ok(protobuf) => {
                                                if let Err(err) = write_half.write_all(&protobuf).await {
                                                    error!("Failed to send response: {err}");
                                                    break;
                                                }
                                            },
                                            Err(err) => {
                                                error!("Failed to encode protobuf: {err}")
                                            }
                                        }
                                    }
                                    Err(_) => break,
                                }
                            }
                        }
                    }
                });
            }
        }
    });

    Ok(incoming_rx)
}
