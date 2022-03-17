mod proto {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/daemon.rs"));
}

pub use proto::*;

pub fn new_diagnostic_message() -> DaemonMessage {
    DaemonMessage {
        id: None,
        no_response: None,
        command: Some(daemon_message::Command::Diagnostic(DiagnosticCommand {
            parts: vec![],
        })),
    }
}

pub fn new_diagnostic_response(
    time_started_epoch: Option<u64>,
    settings_watcher_status: Option<diagnostic_response::SettingsWatcherStatus>,
    websocket_status: Option<diagnostic_response::WebsocketStatus>,
    unix_socket_status: Option<diagnostic_response::UnixSocketStatus>,
) -> daemon_response::Response {
    daemon_response::Response::Diagnostic(DiagnosticResponse {
        time_started_epoch,
        settings_watcher_status,
        websocket_status,
        unix_socket_status,
    })
}
