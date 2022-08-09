use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("telemetry is disabled")]
    TelemetryDisabled,
    #[error(transparent)]
    DefaultsError(#[from] fig_auth::defaults::DefaultsError),
    #[error(transparent)]
    Request(#[from] fig_request::Error),
    #[error(transparent)]
    SettingsError(#[from] fig_settings::Error),
    #[error(transparent)]
    IpcError(#[from] fig_ipc::Error),
}
