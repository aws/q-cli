pub mod cognito;
pub mod password;

pub mod defaults;
mod reqwest_client;

use std::time::Duration;

use cfg_if::cfg_if;
use cognito::get_client;
pub use cognito::Credentials;
pub use defaults::{
    get_default,
    remove_default,
    set_default,
};
pub use reqwest_client::{
    reqwest_client,
    user_agent,
};
pub use thiserror::Error;
use tokio::time::error::Elapsed;

pub const CLIENT_ID: &str = "hkinciohdp1i7h0imdk63a4bv";
pub const REGION: &str = "us-east-1";

const TIMEOUT_DURATION: Duration = Duration::from_secs(10);

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Cognito(#[from] cognito::Error),
    #[error("no access token set")]
    NoAccessToken,
    #[error("could not read from defaults")]
    Defaults(#[from] defaults::DefaultsError),
    #[error("timeout")]
    Timeout(#[from] Elapsed),
}

pub fn logout() -> Result<(), Error> {
    let creds = Credentials::default();
    creds.save_credentials()?;
    Ok(())
}

pub async fn get_token() -> Result<String, Error> {
    let mut creds = Credentials::load_credentials()?;
    if creds.is_expired() {
        let aws_client = get_client()?;
        tokio::time::timeout(TIMEOUT_DURATION, creds.refresh_credentials(&aws_client, None))
            .await?
            .map_err(cognito::Error::from)?;
        creds.save_credentials()?;
    }

    match (creds.get_access_token(), creds.get_refresh_token()) {
        (None, _) => Err(Error::NoAccessToken),
        // TODO: Migrate those with only `access_token`
        (Some(_), None) => Ok(creds.encode()),
        (Some(_), Some(_)) => Ok(creds.encode()),
    }
}

pub fn get_email() -> Option<String> {
    cfg_if! {
        if #[cfg(target_os = "macos")] {
            Credentials::load_credentials()
                .map(|creds| creds.email)
                .ok()
                .or_else(|| Some(get_default("userEmail").ok()))?
        } else {
            Credentials::load_credentials().ok().and_then(|creds| creds.email)
        }
    }
}

#[must_use]
pub fn is_logged_in() -> bool {
    get_email().is_some()
}
