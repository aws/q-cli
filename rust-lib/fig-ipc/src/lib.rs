//! Utiities for IPC with Mac App
pub mod command;
pub mod daemon;
pub mod figterm;
pub mod hook;
pub mod util;

use anyhow::Result;
use bytes::BytesMut;
use fig_proto::FigProtobufEncodable;
use prost::Message;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::{
    fs::remove_file,
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::{UnixListener, UnixStream},
};

/// Get path to "$TMPDIR/fig.socket"
pub fn get_fig_socket_path() -> PathBuf {
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

pub async fn send_message<M, S>(stream: &mut S, message: M) -> Result<()>
where
    M: Message + FigProtobufEncodable,
    S: AsyncWriteExt + Unpin,
{
    let encoded_message = message.encode_fig_protobuf()?;

    stream.write_all(&encoded_message).await?;
    Ok(())
}

pub async fn recv_message<T, S>(stream: &mut S) -> Result<T>
where
    T: Message + Default,
    S: AsyncReadExt + Unpin,
{
    let mut buffer = BytesMut::new();
    stream.read_buf(&mut buffer).await?;

    while buffer.len() < 10 {
        stream.read_buf(&mut buffer).await?;
    }

    let proto_type = match buffer.split_to(10).as_ref() {
        b"\x1b@fig-pbuf" => Ok(()),
        _ => Err(anyhow::anyhow!(
            "Invalid message header: {:?}",
            String::from_utf8_lossy(buffer.as_ref())
        )),
    };

    while buffer.len() < 8 {
        stream.read_buf(&mut buffer).await?;
    }

    let msg_size = u64::from_be_bytes(buffer.split_to(8).as_ref().try_into()?);

    while buffer.len() < msg_size as usize {
        stream.read_buf(&mut buffer).await?;
    }

    proto_type?;

    T::decode(buffer.split_to(msg_size as usize).as_ref()).map_err(|err| anyhow::anyhow!(err))
}

pub async fn send_recv_message<M, R, S>(stream: &mut S, message: M, timeout: Duration) -> Result<R>
where
    M: Message + FigProtobufEncodable,
    R: Message + Default,
    S: AsyncReadExt + AsyncWriteExt + Unpin,
{
    send_message(stream, message).await?;
    tokio::time::timeout(timeout, recv_message(stream)).await?
}

pub async fn create_socket_listen(session_id: impl AsRef<str>) -> io::Result<UnixListener> {
    let socket_path: PathBuf = [
        Path::new("/tmp"),
        Path::new(&format!("figterm-{}.socket", session_id.as_ref())),
    ]
    .into_iter()
    .collect();

    // Remove the socket so we can create a new one
    if socket_path.exists() {
        remove_file(&socket_path).await?
    }

    Ok(UnixListener::bind(&socket_path)?)
}
