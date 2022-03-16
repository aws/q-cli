pub mod cognito;
pub mod password;

pub use cognito::Credentials;

use anyhow::Result;
use cognito::get_client;
use std::{ffi::OsStr, process::Command};

pub const CLIENT_ID: &str = "hkinciohdp1i7h0imdk63a4bv";

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
    match Credentials::load_credentials() {
        Ok(mut creds) => {
            if creds.is_expired() {
                let aws_client = get_client()?;
                creds.refresh_credentials(&aws_client, CLIENT_ID).await?;
                creds.save_credentials()?;
            }

            Ok(creds.encode())
        }
        _ => {
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
                        experation_time: None,
                    };

                    let client = get_client()?;
                    creds.refresh_credentials(&client, CLIENT_ID).await?;
                    creds.save_credentials()?;

                    Ok(creds.encode())
                }
                Err(_) => {
                    // Cotter token
                    Ok(access_token)
                }
            }
        }
    }
}

pub fn get_email() -> Option<String> {
    Credentials::load_credentials()
        .map(|creds| creds.email)
        .ok()
        .or_else(|| Some(get_default("userEmail").ok()))?
}
