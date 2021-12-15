//! Utiities for IPC with Mac App

pub mod hooks;

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use crate::proto;

use anyhow::Result;
use tokio::{io::AsyncWriteExt, net::UnixStream};

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
