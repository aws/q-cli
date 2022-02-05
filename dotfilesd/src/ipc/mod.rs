//! Utiities for IPC with Mac App
pub mod command;
pub mod hook;

use std::{
    io,
    path::{Path, PathBuf},
    time::Duration,
};

use crate::proto::{local, FigProtobufEncodable};
use bytes::BytesMut;

use anyhow::Result;
use prost::Message;
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

pub async fn send_command(
    connection: &mut UnixStream,
    command: local::command::Command,
) -> Result<()> {
    let message = local::LocalMessage {
        r#type: Some(local::local_message::Type::Command(local::Command {
            id: None,
            no_response: Some(false),
            command: Some(command),
        })),
    };

    let encoded_message = message.encode_fig_protobuf()?;

    connection.write_all(&encoded_message).await?;
    Ok(())
}

pub async fn send_recv_command(
    connection: &mut UnixStream,
    command: local::command::Command,
) -> Result<local::CommandResponse> {
    send_command(connection, command).await?;

    tokio::time::timeout(Duration::from_secs(2), connection.readable()).await??;
    let mut proto_type: [u8; 10] = [0; 10];
    let proto_type = match connection.try_read(&mut proto_type) {
        Ok(10) => std::str::from_utf8(&proto_type)?,
        Ok(n) => anyhow::bail!("Read {} bytes for message type", n),
        Err(e) => anyhow::bail!("Could not get message type {}", e),
    };

    let mut msg_size: [u8; 8] = [0; 8];
    connection.readable().await?;
    let msg_size = match connection.try_read(&mut msg_size) {
        Ok(8) => u64::from_be_bytes(msg_size),
        Ok(n) => anyhow::bail!("Read {} bytes for message size", n),
        Err(e) => anyhow::bail!("Could not get message size {}", e),
    };

    let mut buf = BytesMut::new();
    let mut bytes_left: usize = usize::try_from(msg_size)?;
    loop {
        connection.readable().await?;
        match connection.try_read_buf(&mut buf) {
            Ok(n) => {
                if bytes_left <= n || n == 0 {
                    break;
                }
                bytes_left -= n;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => anyhow::bail!(e),
        }
    }

    if proto_type != "\x1b@fig-pbuf" {
        anyhow::bail!("Unexpected message type");
    }

    local::CommandResponse::decode(buf.as_ref()).map_err(|err| anyhow::anyhow!(err))
}

pub async fn send_hook_to_socket(hook: local::hook::Hook) -> Result<()> {
    let path = get_socket_path();
    let mut conn = connect_timeout(&path, Duration::from_secs(3)).await?;
    send_hook(&mut conn, hook).await
}

pub async fn send_command_to_socket(command: local::command::Command) -> Result<()> {
    let path = get_socket_path();
    let mut conn = connect_timeout(&path, Duration::from_secs(3)).await?;
    send_command(&mut conn, command).await
}

pub async fn send_recv_command_to_socket(
    command: local::command::Command,
) -> Result<local::CommandResponse> {
    let path = get_socket_path();
    let mut conn = connect_timeout(&path, Duration::from_secs(3)).await?;
    send_recv_command(&mut conn, command).await
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
