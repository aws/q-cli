use std::time::Duration;

use fig_proto::local;
use fig_util::directories;
use tokio::net::UnixStream;

use super::{
    connect_timeout,
    send_message,
};

/// Send a hook using a system socket
pub async fn send_hook(connection: &mut UnixStream, hook: local::Hook) -> Result<(), crate::SendError> {
    let message = local::LocalMessage {
        r#type: Some(local::local_message::Type::Hook(hook)),
    };

    send_message(connection, message).await
}

/// Send a hook directly to the Fig socket
pub async fn send_hook_to_socket(hook: local::Hook) -> Result<(), crate::Error> {
    let path = directories::fig_socket_path()?;
    let mut conn = connect_timeout(&path, Duration::from_secs(3)).await?;
    Ok(send_hook(&mut conn, hook).await?)
}
