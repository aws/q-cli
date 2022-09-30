pub use crate::proto::daemon::*;

pub fn new_diagnostic_message() -> DaemonMessage {
    use diagnostic_command::DiagnosticPart;

    DaemonMessage {
        id: None,
        no_response: None,
        command: Some(daemon_message::Command::Diagnostic(DiagnosticCommand {
            parts: vec![
                DiagnosticPart::TimeStartedEpoch.into(),
                DiagnosticPart::SettingsWatcherStatus.into(),
                DiagnosticPart::WebsocketStatus.into(),
                DiagnosticPart::SystemSocketStatus.into(),
            ],
        })),
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

pub fn new_ping_command() -> DaemonMessage {
    DaemonMessage {
        id: None,
        no_response: None,
        command: Some(daemon_message::Command::Ping(())),
    }
}

pub fn new_ping_response() -> daemon_response::Response {
    daemon_response::Response::Pong(())
}

pub fn new_quit_command() -> DaemonMessage {
    DaemonMessage {
        id: None,
        no_response: None,
        command: Some(daemon_message::Command::Quit(())),
    }
}

pub fn new_quit_response() -> daemon_response::Response {
    daemon_response::Response::Quit(())
}

pub fn new_log_level_command(level: String) -> DaemonMessage {
    DaemonMessage {
        id: None,
        no_response: None,
        command: Some(daemon_message::Command::LogLevel(LogLevelCommand { level })),
    }
}

pub fn new_log_level_response(result: Result<String, String>) -> daemon_response::Response {
    match result {
        Ok(level) => daemon_response::Response::LogLevel(LogLevelResponse {
            status: log_level_response::Status::Ok.into(),
            old_level: Some(level),
            error: None,
        }),
        Err(err) => daemon_response::Response::LogLevel(LogLevelResponse {
            status: log_level_response::Status::Error.into(),
            old_level: None,
            error: Some(err),
        }),
    }
}
