//! Utiities for IPC with Mac App
pub mod command;

use std::{
    path::{Path, PathBuf},
    time::Duration,
};


use crate::proto::{local, FigProtobufEncodable};

use anyhow::Result;
// use bytes::{Bytes, BytesMut};
// use prost::Message;
use tokio::{
    fs::remove_file,
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
    let conn = tokio::time::timeout(timeout, UnixStream::connect(socket)).await??;

    // When on macOS after the socket connection is made a breif delay is required
    // Not sure why, but this is a workaround
    #[cfg(target_os = "macos")]
    tokio::time::sleep(Duration::from_millis(2)).await;

    Ok(conn)
}

/// Send a hook using a Unix socket
pub async fn send_hook(connection: &mut UnixStream, hook: local::hook::Hook) -> Result<()> {
    let message = local::LocalMessage {
        r#type: Some(local::local_message::Type::Hook(local::Hook {
            hook: Some(hook),
        })),
    };

    let encoded_message = message.encode_fig_protobuf()?;

    connection.write_all(&encoded_message).await?;
    Ok(())
}

pub async fn create_socket_listen(session_id: impl AsRef<str>) -> Result<UnixListener> {
    let session_id_str = session_id.as_ref().split(':').last().unwrap();

    let socket_path: PathBuf = [
        Path::new("/tmp"),
        Path::new(&format!("figterm-{}.socket", session_id_str)),
    ]
    .into_iter()
    .collect();

    // Remove the socket so we can create a new one
    if socket_path.exists() {
        remove_file(&socket_path).await?
    }

    Ok(UnixListener::bind(&socket_path)?)
}
