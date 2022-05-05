pub mod cognito;
pub mod password;

use std::ffi::OsStr;
use std::process::Command;
use std::time::Duration;

use anyhow::Result;
use cognito::get_client;
pub use cognito::Credentials;

pub const CLIENT_ID: &str = "hkinciohdp1i7h0imdk63a4bv";
const TIMEOUT_DURATION: Duration = Duration::from_secs(10);

pub fn get_default(key: impl AsRef<OsStr>) -> Result<String> {
    let output = Command::new("defaults")
        .arg("read")
        .arg("com.mschrage.fig")
        .arg(key)
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("defaults read failed"));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().into())
}

pub fn set_default(key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> Result<()> {
    let output = Command::new("defaults")
        .arg("write")
        .arg("com.mschrage.fig")
        .arg(key)
        .arg(value)
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("defaults write failed"));
    }

    Ok(())
}

pub async fn refresh_credentals() -> Result<Credentials> {
    let mut creds = Credentials::load_credentials()?;
    let aws_client = get_client()?;
    creds.refresh_credentials(&aws_client, CLIENT_ID).await?;
    creds.save_credentials()?;
    Ok(creds)
}

pub fn remove_default(key: impl AsRef<OsStr>) -> Result<()> {
    let output = Command::new("defaults")
        .arg("delete")
        .arg("com.mschrage.fig")
        .arg(key)
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("defaults write failed"));
    }

    Ok(())
}

pub async fn get_token() -> Result<String> {
    if let Ok(mut creds) = Credentials::load_credentials() {
        if creds.is_expired() {
            let aws_client = get_client()?;
            tokio::time::timeout(TIMEOUT_DURATION, creds.refresh_credentials(&aws_client, CLIENT_ID)).await??;
            creds.save_credentials()?;
        }

        Ok(creds.encode())
    } else {
        let access_token = get_default("access_token")?;
        let email = get_default("userEmail")?;

        match get_default("refresh_token") {
            Ok(refresh_token) => {
                // Aws cognito token
                let id_token = get_default("id_token")?;

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
            },
            Err(_) => {
                // Cotter token
                Ok(access_token)
            },
        }
    }
}

#[must_use]
pub fn get_email() -> Option<String> {
    Credentials::load_credentials()
        .map(|creds| creds.email)
        .ok()
        .or_else(|| Some(get_default("userEmail").ok()))?
}

#[must_use]
pub fn is_logged_in() -> bool {
    get_email().is_some()
}
