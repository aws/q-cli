use anyhow::Result;
use fig_proto::local;
use std::time::Duration;
use tokio::net::UnixStream;

use super::{connect_timeout, get_fig_socket_path, send_message};

/// Send a hook using a Unix socket
pub async fn send_hook(connection: &mut UnixStream, hook: local::Hook) -> Result<()> {
    let message = local::LocalMessage {
        r#type: Some(local::local_message::Type::Hook(hook)),
    };

    send_message(connection, message).await
}

/// Send a hook directly to the Fig socket
pub async fn send_hook_to_socket(hook: local::Hook) -> Result<()> {
    let path = get_fig_socket_path();
    let mut conn = connect_timeout(&path, Duration::from_secs(3)).await?;
    send_hook(&mut conn, hook).await
}
