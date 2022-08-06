use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("telemetry is disabled")]
    TelemetryDisabled,
    #[error(transparent)]
    DefaultsError(#[from] fig_auth::defaults::DefaultsError),
    #[error(transparent)]
    Request(#[from] fig_request::Error),
    // TODO(grant): remove other varient
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
