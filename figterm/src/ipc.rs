use std::{time::Duration, path::{Path, PathBuf}};

use crate::local;

use anyhow::Result;
use prost::Message;
use tokio::{io::AsyncWriteExt, net::UnixStream};

fn get_socket_path() -> PathBuf {
    [std::env::temp_dir(), "fig.socket".into()].into_iter().collect()
}

async fn connect_timeout(socket: impl AsRef<Path>, timeout: Duration) -> Result<UnixStream> {
    Ok(tokio::time::timeout(timeout, UnixStream::connect(socket)).await??)
}

fn message_to_packet(message: local::LocalMessage) -> Vec<u8> {
    let mut packet: Vec<u8> = Vec::new();

    let encoded_message = message.encode_to_vec();

    packet.extend(b"\x1b@fig-pbuf");
    packet.extend(encoded_message.len().to_be_bytes());
    packet.extend(encoded_message);

    packet
}

async fn send_hook(connection: &mut UnixStream, hook: local::hook::Hook) -> Result<()> {
    let message = local::LocalMessage {
        r#type: Some(local::local_message::Type::Hook(local::Hook {
            hook: Some(hook),
        })),
    };

    let encoded_message = message_to_packet(message);

    connection.write_all(&encoded_message).await?;
    Ok(())
}
