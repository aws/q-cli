mod proto {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/daemon.rs"));
}

pub use proto::*;

pub fn new_diagnostic_message() -> DaemonMessage {
    DaemonMessage {
        id: None,
        no_response: None,
        command: Some(daemon_message::Command::Diagnostic(DiagnosticCommand {})),
    }
}

pub fn new_diagnostic_response(
    time_started_epoch: u64,
    settings_watcher_status: diagnostic_response::SettingsWatcherStatus,
    websocket_status: diagnostic_response::WebsocketStatus,
) -> DaemonResponse {
    DaemonResponse {
        id: None,
        response: Some(daemon_response::Response::Diagnostic(DiagnosticResponse {
            time_started_epoch,
            settings_watcher_status: settings_watcher_status.into(),
            websocket_status: websocket_status.into(),
        })),
    }
}
