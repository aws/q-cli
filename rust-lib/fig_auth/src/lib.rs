pub mod cognito;
pub mod password;

#[cfg(target_os = "macos")]
pub mod defaults;

use std::time::Duration;

use anyhow::Result;
use cfg_if::cfg_if;
use cognito::get_client;
pub use cognito::Credentials;
#[cfg(target_os = "macos")]
pub use defaults::{
    get_default,
    remove_default,
    set_default,
};

pub const CLIENT_ID: &str = "hkinciohdp1i7h0imdk63a4bv";
const TIMEOUT_DURATION: Duration = Duration::from_secs(10);

pub async fn refresh_credentals() -> Result<Credentials> {
    let mut creds = Credentials::load_credentials()?;
    let aws_client = get_client()?;
    creds.refresh_credentials(&aws_client, CLIENT_ID).await?;
    creds.save_credentials()?;
    Ok(creds)
}

pub fn logout() -> Result<()> {
    let creds = Credentials::default();
    creds.save_credentials()?;
    Ok(())
}

async fn get_credentials_file_token() -> Result<String> {
    let mut creds = Credentials::load_credentials()?;
    if creds.is_expired() {
        let aws_client = get_client()?;
        tokio::time::timeout(TIMEOUT_DURATION, creds.refresh_credentials(&aws_client, CLIENT_ID)).await??;
        creds.save_credentials()?;
    }

    Ok(creds.encode())
}

pub async fn get_token() -> Result<String> {
    cfg_if! {
        if #[cfg(target_os = "macos")] {
            match get_credentials_file_token().await {
                Ok(token) => Ok(token),
                Err(_) => {
                    let access_token = get_default("access_token")?;
                    let refresh_token = get_default("refresh_token")?;
                    let id_token = get_default("id_token")?;
                    let email = get_default("userEmail")?;

                    let mut creds = Credentials {
                        email: Some(email),
                        id_token: Some(id_token),
                        access_token: Some(access_token),
                        refresh_token: Some(refresh_token),
                        expiration_time: None,
                    };

                    let aws_client = get_client()?;
                    tokio::time::timeout(TIMEOUT_DURATION, creds.refresh_credentials(&aws_client, CLIENT_ID)).await??;
                    creds.save_credentials()?;

                    Ok(creds.encode())
                }
            }
        } else {
            get_credentials_file_token().await
        }
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
