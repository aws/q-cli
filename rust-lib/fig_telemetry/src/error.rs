use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("telemetry is disabled")]
    TelemetryDisabled,
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    AuthError(#[from] fig_auth::Error),
    #[error(transparent)]
    DefaultsError(#[from] fig_auth::defaults::DefaultsError),
    // TODO(grant): remove other varient
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
