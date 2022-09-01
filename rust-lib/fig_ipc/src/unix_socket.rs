use std::path::Path;
use std::time::Duration;

use tokio::net::UnixStream;
use tracing::{
    error,
    trace,
};

use crate::{
    BufferedReader,
    ConnectError,
};

/// Connects to a unix socket
pub async fn socket_connect(socket: impl AsRef<Path>) -> Result<UnixStream, ConnectError> {
    let socket = socket.as_ref();
    let stream = match UnixStream::connect(socket).await {
        Ok(stream) => stream,
        Err(err) => {
            error!(%err, ?socket, "Failed to connect");
            return Err(err.into());
        },
    };

    // When on macOS after the socket connection is made a brief delay is required
    // Not sure why, so this is a workaround
    #[cfg(target_os = "macos")]
    tokio::time::sleep(Duration::from_millis(2)).await;

    trace!(?socket, "Connected");

    Ok(stream)
}

/// Connects to a unix socket with a timeout
pub async fn socket_connect_timeout(socket: impl AsRef<Path>, timeout: Duration) -> Result<UnixStream, ConnectError> {
    let socket = socket.as_ref();
    match tokio::time::timeout(timeout, socket_connect(&socket)).await {
        Ok(Ok(conn)) => Ok(conn),
        Ok(Err(err)) => Err(err),
        Err(_) => {
            error!(?socket, ?timeout, "Timeout while connecting");
            Err(ConnectError::Timeout)
        },
    }
}

pub type BufferedUnixStream = BufferedReader<UnixStream>;

impl BufferedUnixStream {
    /// Connect to a unix socket
    pub async fn connect(socket: impl AsRef<Path>) -> Result<Self, ConnectError> {
        Ok(Self::new(socket_connect(socket).await?))
    }

    /// Connect to a unix socket with a timeout
    pub async fn connect_timeout(socket: impl AsRef<Path>, timeout: Duration) -> Result<Self, ConnectError> {
        Ok(Self::new(socket_connect_timeout(socket, timeout).await?))
    }
}
