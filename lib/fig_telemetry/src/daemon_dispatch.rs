use std::time::Duration;

use fig_ipc::{BufferedUnixStream, SendMessage};
use fig_util::directories;

use crate::util::telemetry_is_disabled;
use crate::{Error, TrackEvent, TrackSource};

async fn send_daemon_message(message: DaemonMessage) -> Result<(), fig_ipc::Error> {
    let daemon_socket_path = directories::daemon_socket_path()?;
    let mut conn = BufferedUnixStream::connect_timeout(daemon_socket_path, Duration::from_secs(1)).await?;
    conn.send_message(message).await?;
    Ok(())
}

pub async fn dispatch_emit_track(event: TrackEvent, enqueue: bool, fallback: bool) -> Result<(), Error> {
    if telemetry_is_disabled() {
        return Err(Error::TelemetryDisabled);
    }

    crate::emit_track(event).await?;
    Ok(())
}
