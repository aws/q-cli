//! Utiities for IPC with Mac App

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use crate::proto;

use anyhow::Result;
use tokio::{
    io::AsyncWriteExt,
    net::{UnixListener, UnixStream},
};

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
pub async fn send_hook(connection: &mut UnixStream, hook: proto::hook::Hook) -> Result<()> {
    let message = proto::LocalMessage {
        r#type: Some(proto::local_message::Type::Hook(proto::Hook {
            hook: Some(hook),
        })),
    };

    let encoded_message = message.to_fig_pbuf();

    connection.write_all(&encoded_message).await?;
    Ok(())
}

pub async fn create_socket_listen(session_id: impl AsRef<str>) -> Result<UnixListener> {
    let path: PathBuf = [
        Path::new("/tmp"),
        Path::new(&format!("figterm-{}.socket", session_id.as_ref())),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn socket_path_test() {
        assert!(get_socket_path().ends_with("fig.socket"))
    }
}
