use std::time::Duration;

use fig_proto::daemon::daemon_message::Command;
use fig_proto::daemon::telemetry_emit_track_command::{
    Property,
    Source,
};
use fig_proto::daemon::{
    DaemonMessage,
    TelemetryEmitTrackCommand,
};

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

pub async fn dispatch_emit_track<'a, I, T>(
    event: impl Into<TrackEvent>,
    source: TrackSource,
    properties: I,
) -> Result<(), Error>
where
    I: IntoIterator<Item = T>,
    T: Into<(&'a str, &'a str)>,
{
    let event = event.into();
    let properties: Vec<(&'a str, &'a str)> = properties.into_iter().map(|prop| prop.into()).collect();

    let message = DaemonMessage {
        id: None,
        no_response: Some(true),
        command: Some(Command::TelemetryEmitTrack(TelemetryEmitTrackCommand {
            event: event.to_string(),
            properties: properties
                .iter()
                .map(|(key, value)| Property {
                    key: (*key).into(),
                    value: (*value).into(),
                })
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
