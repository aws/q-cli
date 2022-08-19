pub mod command;
pub mod hook;

use std::fmt::Debug;
use std::io::{
    Cursor,
    Write,
};
#[cfg(unix)]
use std::os::unix::net::UnixStream as SyncUnixStream;
use std::path::Path;
use std::time::Duration;

use bytes::BytesMut;
use fig_proto::prost::Message;
use fig_proto::{
    FigMessage,
    FigProtobufEncodable,
    ReflectMessage,
};
use thiserror::Error;
use tokio::io::{
    self,
    AsyncRead,
    AsyncReadExt,
    AsyncWrite,
    AsyncWriteExt,
};
use tokio::net::UnixStream;
use tracing::{
    error,
    trace,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Connect(#[from] ConnectError),
    #[error(transparent)]
    Send(#[from] SendError),
    #[error(transparent)]
    Recv(#[from] RecvError),
    #[error("timeout")]
    Timeout,
    #[error(transparent)]
    Dir(#[from] fig_util::directories::DirectoryError),
}

#[derive(Debug, Error)]
pub enum ConnectError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("timeout connecting to socket")]
    Timeout,
}

pub async fn connect(socket: impl AsRef<Path>) -> Result<UnixStream, ConnectError> {
    let socket = socket.as_ref();
    let conn = match UnixStream::connect(socket).await {
        Ok(conn) => conn,
        Err(err) => {
            error!("Failed to connect to {socket:?}: {err}");
            return Err(err.into());
        },
    };

    #[cfg(target_os = "macos")]
    // When on macOS after the socket connection is made a brief delay is required
    // Not sure why, so this is a workaround
    tokio::time::sleep(Duration::from_millis(2)).await;

    trace!("Connected to {socket:?}");

    Ok(conn)
}

/// Connect to a system socket with a timeout
pub async fn connect_timeout(socket: impl AsRef<Path>, timeout: Duration) -> Result<UnixStream, ConnectError> {
    let socket = socket.as_ref();
    let conn = match tokio::time::timeout(timeout, UnixStream::connect(socket)).await {
        Ok(Ok(conn)) => conn,
        Ok(Err(err)) => {
            error!("Failed to connect to {socket:?}: {err}");
            return Err(err.into());
        },
        Err(_) => {
            error!("Timeout while connecting to {socket:?}");
            return Err(ConnectError::Timeout);
        },
    };

    #[cfg(target_os = "macos")]
    // When on macOS after the socket connection is made a brief delay is required
    // Not sure why, so this is a workaround
    tokio::time::sleep(Duration::from_millis(2)).await;

    trace!("Connected to {socket:?}");

    Ok(conn)
}

/// Connect to `socket` synchronously without a timeout
#[cfg(unix)]
pub fn connect_sync(socket: impl AsRef<Path>) -> Result<SyncUnixStream, ConnectError> {
    let socket = socket.as_ref();
    let conn = match SyncUnixStream::connect(socket) {
        Ok(conn) => conn,
        Err(err) => {
            error!("Failed to connect to {socket:?}: {err}");
            return Err(err.into());
        },
    };

    trace!("Connected to {socket:?}");

    // When on macOS after the socket connection is made a brief delay is required
    // Not sure why, but this is a workaround
    #[cfg(target_os = "macos")]
    std::thread::sleep(std::time::Duration::from_millis(2));

    Ok(conn)
}

#[derive(Debug, Error)]
pub enum SendError {
    #[error(transparent)]
    Encode(#[from] fig_proto::FigMessageEncodeError),
    #[error(transparent)]
    Io(#[from] io::Error),
}

pub async fn send_message<M, S>(stream: &mut S, message: M) -> Result<(), SendError>
where
    M: FigProtobufEncodable,
    S: AsyncWrite + Unpin,
{
    let encoded_message = match message.encode_fig_protobuf() {
        Ok(encoded_message) => encoded_message,
        Err(err) => {
            error!("Failed to encode message: {err}");
            return Err(err.into());
        },
    };

    match stream.write_all(&encoded_message).await {
        Ok(_) => {
            trace!("Sent message: {message:?}");
            Ok(())
        },
        Err(err) => Err(err.into()),
    }
}

pub fn send_message_sync<M, S>(stream: &mut S, message: M) -> Result<(), SendError>
where
    M: FigProtobufEncodable,
    S: Write,
{
    let encoded_message = match message.encode_fig_protobuf() {
        Ok(encoded_message) => encoded_message,
        Err(err) => {
            error!("Failed to encode message: {err}");
            return Err(err.into());
        },
    };

    match stream.write_all(&encoded_message) {
        Ok(()) => {
            trace!("Sent message: {message:?}");
        },
        Err(err) => {
            error!("Failed to write message: {err}");
            return Err(err.into());
        },
    };

    Ok(())
}

#[derive(Debug, Error)]
pub enum RecvError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Parse(#[from] fig_proto::FigMessageParseError),
    #[error(transparent)]
    Decode(#[from] fig_proto::FigMessageDecodeError),
}

impl RecvError {
    pub fn is_disconnect(&self) -> bool {
        if let RecvError::Io(io) = self {
            #[cfg(windows)]
            {
                use windows_sys::Win32::Networking::WinSock::WSAECONNRESET;
                if let Some(WSAECONNRESET) = io.raw_os_error() {
                    return true;
                }
            }
            matches!(io.kind(), std::io::ErrorKind::ConnectionAborted)
        } else {
            false
        }
    }
}

pub async fn recv_message<T, S>(stream: &mut S) -> Result<Option<T>, RecvError>
where
    T: Message + ReflectMessage + Default,
    S: AsyncRead + Unpin,
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
                    return Err(io::Error::from(io::ErrorKind::ConnectionReset).into());
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
            Ok(message) => return Ok(Some(message.decode()?)),
            // If the message is incomplete, read more into the buffer
            Err(fig_proto::FigMessageParseError::Incomplete) => {
                read_buffer!();
            },
            // On any other error, return the error
            Err(err) => {
                return Err(err.into());
            },
        }
    }
}

pub async fn send_recv_message<M, R, S>(stream: &mut S, message: M) -> Result<Option<R>, Error>
where
    M: FigProtobufEncodable,
    R: Message + ReflectMessage + Default,
    S: AsyncReadExt + AsyncWriteExt + Unpin,
{
    send_message(stream, message).await?;
    Ok(recv_message(stream).await?)
}

pub async fn send_recv_message_timeout<M, R, S>(
    stream: &mut S,
    message: M,
    timeout: Duration,
) -> Result<Option<R>, Error>
where
    M: FigProtobufEncodable,
    R: Message + ReflectMessage + Default,
    S: AsyncReadExt + AsyncWriteExt + Unpin,
{
    send_message(stream, message).await?;
    Ok(match tokio::time::timeout(timeout, recv_message(stream)).await {
        Ok(result) => result?,
        Err(_) => {
            error!("Timeout while receiving message");
            return Err(Error::Timeout);
        },
    })
}
