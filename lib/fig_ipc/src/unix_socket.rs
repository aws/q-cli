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

    #[cfg(unix)]
    if let Some(parent) = socket.parent() {
        use std::os::unix::fs::PermissionsExt;
        let mode = parent.metadata()?.permissions().mode();
        if !validate_mode_bits(mode) {
            error!(?socket, mode, "Socket folder permissions are not 0o700");
            return Err(ConnectError::IncorrectSocketPermissions);
        }
    }

    let stream = match UnixStream::connect(socket).await {
        Ok(stream) => stream,
        Err(err) => {
            error!(%err, ?socket, "Failed to connect");
            return Err(err.into());
        },
    };

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

fn validate_mode_bits(mode: u32) -> bool {
    mode & 0o777 == 0o700
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

#[cfg(test)]
mod tests {
    use super::*;

    /// If this test fails, we need to reevaluate the permissions model design around our sockets
    /// and double check with security
    #[test]
    fn test_socket_folder_mode() {
        assert!(validate_mode_bits(0o700));

        for i in 0..0o700 {
            assert!(!validate_mode_bits(i));
        }

        for i in 0o701..0o777 {
            assert!(!validate_mode_bits(i));
        }
    }
}
