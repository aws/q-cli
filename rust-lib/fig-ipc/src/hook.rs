use anyhow::Result;
use fig_proto::local;
use std::time::Duration;

use super::{connect_timeout, get_fig_socket_path, send_message};

cfg_if! {
    if #[cfg(unix)] {
        use tokio::net::UnixStream;
    } else if #[cfg(windows)] {
        use tokio::net::windows::named_pipe::NamedPipeClient;
    }
}

/// Send a hook using a Unix socket
#[cfg(unix)]
pub async fn send_hook(connection: &mut UnixStream, hook: local::Hook) -> Result<()> {
    let message = local::LocalMessage {
        r#type: Some(local::local_message::Type::Hook(hook)),
    };

    send_message(connection, message).await
}

/// Send a hook using a windows named pipe
#[cfg(windows)]
pub async fn send_hook(connection: &mut NamedPipeClient, hook: local::Hook) -> Result<()> {
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
