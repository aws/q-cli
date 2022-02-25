//! Utiities for IPC with Mac App
pub mod command;
pub mod daemon;
pub mod figterm;
pub mod hook;
pub mod util;

use anyhow::{bail, Result};
use bytes::{Bytes, BytesMut};
use fig_proto::FigProtobufEncodable;
use prost::Message;
use std::fmt::Debug;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use thiserror::Error;
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};
use tracing::{error, trace};

/// Get path to "$TMPDIR/fig.socket"
pub fn get_fig_socket_path() -> PathBuf {
    [std::env::temp_dir().as_path(), Path::new("fig.socket")]
        .into_iter()
        .collect()
}

/// Connect to `socket` with a timeout
pub async fn connect_timeout(socket: impl AsRef<Path>, timeout: Duration) -> Result<UnixStream> {
    let conn = match tokio::time::timeout(timeout, UnixStream::connect(socket.as_ref())).await {
        Ok(Ok(conn)) => conn,
        Ok(Err(err)) => {
            error!("Failed to connect to {:?}: {}", socket.as_ref(), err);
            bail!("Failed to connect to {:?}: {}", socket.as_ref(), err);
        }
        Err(_) => {
            error!("Timeout while connecting to {:?}", socket.as_ref());
            bail!("Timeout while connecting to {:?}", socket.as_ref());
        }
    };

    trace!("Connected to {:?}", socket.as_ref());

    // When on macOS after the socket connection is made a breif delay is required
    // Not sure why, but this is a workaround
    #[cfg(target_os = "macos")]
    tokio::time::sleep(Duration::from_millis(2)).await;

    Ok(conn)
}

pub async fn send_message<M, S>(stream: &mut S, message: M) -> Result<()>
where
    M: FigProtobufEncodable,
    S: AsyncWriteExt + Unpin,
{
    let encoded_message = match message.encode_fig_protobuf() {
        Ok(encoded_message) => encoded_message,
        Err(err) => {
            error!("Failed to encode message: {}", err);
            bail!("Failed to encode message: {}", err);
        }
    };

    match stream.write_all(&encoded_message).await {
        Ok(()) => {
            trace!("Sent message: {:?}", message);
        }
        Err(err) => {
            error!("Failed to write message: {}", err);
            bail!("Failed to write message: {}", err);
        }
    };

    Ok(())
}

#[derive(Debug, Error)]
pub enum RecvError {
    #[error("Failed to read from stream: {0}")]
    Io(#[from] io::Error),
    #[error("Failed to decode message: {0}")]
    Decode(#[from] prost::DecodeError),
    #[error("Invalid message header {0}")]
    InvalidMessageType(String),
    #[error("Invalid message length {0:?}")]
    InvalidMessageLength(Bytes),
}

pub async fn recv_message<T, S>(stream: &mut S) -> Result<T, RecvError>
where
    T: Message + Default,
    S: AsyncReadExt + Unpin,
{
    let mut buffer = BytesMut::new();
    if let Err(err) = stream.read_buf(&mut buffer).await {
        error!("Failed to read from stream: {}", err);
        return Err(RecvError::Io(err));
    }

    while buffer.len() < 10 {
        if let Err(err) = stream.read_buf(&mut buffer).await {
            error!("Failed to read from stream: {}", err);
            return Err(RecvError::Io(err));
        }
    }

    let proto_type = match buffer.split_to(10).as_ref() {
        b"\x1b@fig-pbuf" => Ok(()),
        buff => Err(RecvError::InvalidMessageType(
            String::from_utf8_lossy(buff).to_string(),
        )),
    };

    while buffer.len() < 8 {
        if let Err(err) = stream.read_buf(&mut buffer).await {
            error!("Failed to read from stream: {}", err);
            return Err(RecvError::Io(err));
        }
    }

    let msg_size = buffer.split_to(8);
    let msg_size = u64::from_be_bytes(match msg_size.as_ref().try_into() {
        Ok(msg_size) => msg_size,
        Err(_) => {
            error!("Invalid message length: {:?}", msg_size);
            return Err(RecvError::InvalidMessageLength(msg_size.freeze()));
        }
    });

    while buffer.len() < msg_size as usize {
        if let Err(err) = stream.read_buf(&mut buffer).await {
            error!("Failed to read from stream: {}", err);
            return Err(RecvError::Io(err));
        }
    }

    if let Err(err) = proto_type {
        error!("Invalid message type: {}", err);
        return Err(err);
    }

    Ok(
        match T::decode(buffer.split_to(msg_size as usize).as_ref()) {
            Ok(message) => {
                trace!("Received message: {:?}", message);
                message
            }
            Err(err) => {
                error!("Failed to decode message: {}", err);
                return Err(RecvError::Decode(err));
            }
        },
    )
}

pub async fn send_recv_message<M, R, S>(stream: &mut S, message: M, timeout: Duration) -> Result<R>
where
    M: Message + FigProtobufEncodable,
    R: Message + Default,
    S: AsyncReadExt + AsyncWriteExt + Unpin,
{
    send_message(stream, message).await?;
    Ok(
        match tokio::time::timeout(timeout, recv_message(stream)).await {
            Ok(result) => result?,
            Err(_) => {
                error!("Timeout while receiving message");
                bail!("Timeout while receiving message");
            }
        },
    )
}
