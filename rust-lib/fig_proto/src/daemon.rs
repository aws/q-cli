mod proto {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/daemon.rs"));
}

pub use proto::*;

pub fn new_diagnostic_message() -> DaemonMessage {
    DaemonMessage {
        id: None,
        no_response: None,
        command: Some(daemon_message::Command::Diagnostic(DiagnosticCommand { parts: vec![] })),
    }
}

pub fn new_diagnostic_response(
    time_started_epoch: Option<u64>,
    settings_watcher_status: Option<diagnostic_response::SettingsWatcherStatus>,
    websocket_status: Option<diagnostic_response::WebsocketStatus>,
    system_socket_status: Option<diagnostic_response::SystemSocketStatus>,
) -> daemon_response::Response {
    daemon_response::Response::Diagnostic(DiagnosticResponse {
        time_started_epoch,
        settings_watcher_status,
        websocket_status,
        system_socket_status,
    })
}

pub fn new_self_update_message() -> DaemonMessage {
    DaemonMessage {
        id: None,
        no_response: None,
        command: Some(daemon_message::Command::SelfUpdate(SelfUpdateCommand {})),
    }
}

pub fn new_self_update_response(success: bool) -> daemon_response::Response {
    daemon_response::Response::SelfUpdate(SelfUpdateResponse {
        status: if success {
            self_update_response::Status::Ok.into()
        } else {
            self_update_response::Status::Error.into()
        },
        error: None,
    })
}

pub fn new_sync_message(sync_type: sync_command::SyncType) -> DaemonMessage {
    DaemonMessage {
        id: None,
        no_response: None,
        command: Some(daemon_message::Command::Sync(SyncCommand {
            r#type: sync_type.into(),
        })),
    }
}

pub fn new_sync_response(result: Result<(), String>) -> daemon_response::Response {
    match result {
        Ok(()) => daemon_response::Response::Sync(SyncResponse {
            status: sync_response::SyncStatus::Ok.into(),
            error: None,
        }),
        Err(err) => daemon_response::Response::Sync(SyncResponse {
            status: sync_response::SyncStatus::Error.into(),
            error: Some(err),
        }),
    }
}
