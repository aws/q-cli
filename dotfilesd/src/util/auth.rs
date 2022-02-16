use fig_auth::{self, get_client, Credentials};

use anyhow::Result;
use base64::encode;
use serde_json::json;
use std::{ffi::OsStr, process::Command};

fn get_default(key: impl AsRef<OsStr>) -> Result<String> {
    Ok(String::from_utf8_lossy(
        &Command::new("defaults")
            .arg("read")
            .arg("com.mschrage.fig")
            .arg(key)
            .output()?
            .stdout,
    )
    .trim()
    .into())
}

pub async fn get_token() -> Result<String> {
    match Credentials::load_credentials() {
        Ok(mut creds) => {
            let aws_client = get_client("")?;

            if creds.is_expired() {
                creds
                    .refresh_credentials(&aws_client, "hkinciohdp1i7h0imdk63a4bv")
                    .await?;
                creds.save_credentials()?;
            }

            Ok(encode(
                json!({
                    "accessToken": creds.access_token,
                    "idToken": creds.id_token
                })
                .to_string(),
            ))
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

                    let client = get_client("")?;
                    creds
                        .refresh_credentials(&client, "hkinciohdp1i7h0imdk63a4bv")
                        .await?;
                    creds.save_credentials()?;

                    Ok(encode(
                        json!({
                            "accessToken": creds.access_token,
                            "idToken": creds.id_token
                        })
                        .to_string(),
                    ))
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
