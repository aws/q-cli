//! Utiities for IPC with Mac App
pub mod command;
pub mod daemon;
pub mod figterm;
pub mod hook;

use anyhow::{bail, Result};
use bytes::BytesMut;
use fig_proto::{FigMessage, FigProtobufEncodable};
use prost::Message;
use std::fmt::Debug;
use std::io::Cursor;
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
    #[error("connection reset by peer")]
    ConnectionReset,
    #[error("failed to read from stream: {0}")]
    Io(#[from] io::Error),
    #[error("failed to decode message: {0}")]
    Decode(#[from] prost::DecodeError),
    #[error("failed to parse message: {0}")]
    Parse(#[from] fig_proto::FigMessageParseError),
}

pub async fn recv_message<T, S>(stream: &mut S) -> Result<Option<T>, RecvError>
where
    T: Message + Default,
    S: AsyncReadExt + Unpin,
{
    let mut buffer = BytesMut::with_capacity(1024);

    macro_rules! read_buffer {
        () => {{
            let n = stream.read_buf(&mut buffer).await?;
            if n == 0 {
                if buffer.is_empty() {
                    // If the buffer is empty, we've reached EOF
                    return Ok(None);
                } else {
                    // If the buffer is not empty, the connection was reset
                    return Err(RecvError::ConnectionReset);
                }
            }
            n
        }};
    }

    // Read into buffer the first time
    read_buffer!();

    loop {
        // Try to parse the message until the buffer is a valid message
        let mut cursor = Cursor::new(buffer.as_ref());
        match FigMessage::parse(&mut cursor) {
            // If the parsed message is valid, return it
            Ok(message) => return Ok(Some(T::decode(message.as_ref())?)),
            // If the message is incomplete, read more into the buffer
            Err(fig_proto::FigMessageParseError::Incomplete) => {
                read_buffer!();
            }
            // On any other error, return the error
            Err(err) => {
                return Err(err.into());
            }
        }
    }
}

pub async fn send_recv_message<M, R, S>(
    stream: &mut S,
    message: M,
    timeout: Duration,
) -> Result<Option<R>>
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
