use std::time::Duration;

use fig_proto::daemon;
use fig_util::directories::daemon_socket_path;

use crate::{
    BufferedUnixStream,
    Error,
    RecvMessage,
    SendMessage,
};

pub async fn send_recv_message(message: daemon::DaemonMessage) -> Result<Option<daemon::DaemonResponse>, crate::Error> {
    let mut conn = BufferedUnixStream::connect_timeout(&daemon_socket_path()?, Duration::from_secs(1)).await?;
    conn.send_message(message).await?;
    Ok(tokio::time::timeout(Duration::from_secs(2), conn.recv_message())
        .await
        .or(Err(Error::Timeout))??)
}
