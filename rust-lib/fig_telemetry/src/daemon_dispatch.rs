use std::time::Duration;

use fig_proto::daemon::daemon_message::Command;
use fig_proto::daemon::telemetry_emit_track_command::Source;
use fig_proto::daemon::{
    DaemonMessage,
    TelemetryEmitTrackCommand,
};
use serde_json::Value;

use crate::util::telemetry_is_disabled;
use crate::{
    Error,
    TrackEvent,
    TrackSource,
};

async fn send_daemon_message(message: DaemonMessage) -> Result<(), Error> {
    let daemon_socket_path = fig_ipc::daemon::get_daemon_socket_path();
    let mut conn = fig_ipc::connect_timeout(daemon_socket_path, Duration::from_secs(1)).await?;
    fig_ipc::send_message(&mut conn, message).await?;
    Ok(())
}

pub async fn dispatch_emit_track<I, K, V>(event: TrackEvent, source: TrackSource, properties: I) -> Result<(), Error>
where
    I: IntoIterator<Item = (K, V)> + Clone,
    K: Into<String>,
    V: Into<Value>,
{
    if telemetry_is_disabled() {
        return Err(Error::TelemetryDisabled);
    }

    let message = DaemonMessage {
        id: None,
        no_response: Some(true),
        command: Some(Command::TelemetryEmitTrack(TelemetryEmitTrackCommand {
            event: event.to_string(),
            properties: properties
                .clone()
                .into_iter()
                .map(|(key, value)| (key.into(), value.into().into()))
                .collect(),
            source: Some(
                match source {
                    TrackSource::App => Source::App,
                    TrackSource::Cli => Source::Cli,
                    TrackSource::Daemon => Source::Daemon,
                }
                .into(),
            ),
        })),
    };

    match send_daemon_message(message).await {
        Ok(()) => Ok(()),
        Err(err) => {
            tracing::error!("Failed to dispatch telemetry event to daemon: {err}");
            crate::emit_track(event, source, properties).await?;
            Ok(())
        },
    }
}
