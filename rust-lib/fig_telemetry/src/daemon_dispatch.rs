use std::time::Duration;

use fig_ipc::{
    BufferedUnixStream,
    SendMessage,
};
use fig_proto::daemon::daemon_message::Command;
use fig_proto::daemon::telemetry_emit_track_command::Source;
use fig_proto::daemon::{
    DaemonMessage,
    TelemetryEmitTrackCommand,
};
use fig_util::directories;

use crate::util::telemetry_is_disabled;
use crate::{
    Error,
    TrackEvent,
    TrackSource,
};

async fn send_daemon_message(message: DaemonMessage) -> Result<(), fig_ipc::Error> {
    let daemon_socket_path = directories::daemon_socket_path()?;
    let mut conn = BufferedUnixStream::connect_timeout(daemon_socket_path, Duration::from_secs(1)).await?;
    conn.send_message(message).await?;
    Ok(())
}

pub async fn dispatch_emit_track(event: TrackEvent, enqueue: bool) -> Result<(), Error> {
    if telemetry_is_disabled() {
        return Err(Error::TelemetryDisabled);
    }

    let message = DaemonMessage {
        id: None,
        no_response: Some(true),
        command: Some(Command::TelemetryEmitTrack(TelemetryEmitTrackCommand {
            event: event.event.to_string(),
            properties: event
                .properties
                .clone()
                .into_iter()
                .map(|(key, value)| (key, value.into()))
                .collect(),
            source: Some(
                match event.source {
                    TrackSource::App => Source::App,
                    TrackSource::Cli => Source::Cli,
                    TrackSource::Daemon => Source::Daemon,
                }
                .into(),
            ),
            enqueue: Some(enqueue),
        })),
    };

    match send_daemon_message(message).await {
        Ok(()) => Ok(()),
        Err(err) => {
            tracing::error!("Failed to dispatch telemetry event to daemon: {err}");
            crate::emit_track(event).await?;
            Ok(())
        },
    }
}
