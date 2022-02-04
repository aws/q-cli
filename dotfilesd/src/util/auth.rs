use std::process::Command;

use base64::encode;
use serde_json::json;

use anyhow::{Context, Result};

use crate::auth::{self, Credentials};

pub async fn get_token() -> Result<String> {
    match Credentials::load_credentials() {
        Ok(mut creds) => {
            let aws_client = auth::get_client("")?;

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
            let token = Command::new("defaults")
                .arg("read")
                .arg("com.mschrage.fig")
                .arg("access_token")
                .output()
                .with_context(|| "Could not read access_token")?;

            Ok(String::from_utf8_lossy(&token.stdout).trim().into())
        }
    }
}

pub fn get_email() -> Option<String> {
    Credentials::load_credentials()
        .map(|creds| creds.email)
        .or_else(|_| {
            let out = Command::new("defaults")
                .arg("read")
                .arg("com.mschrage.fig")
                .arg("userEmail")
                .output()?;

            let email = String::from_utf8(out.stdout)?.trim().into();

            anyhow::Ok(Some(email))
        })
        .ok()?
}
