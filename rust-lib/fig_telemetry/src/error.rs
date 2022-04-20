use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Telemetry has been disabled")]
    TelemetryDisabled,
    #[error("Error making telemetry request")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Error with the auth service")]
    AuthError(#[from] anyhow::Error),
}
