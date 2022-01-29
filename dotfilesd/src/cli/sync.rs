//! Sync of dotfiles

use std::{io::Write, process::Command};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::try_join;

use crate::{auth::Credentials, util::shell::Shell};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DotfilesSourceRequest {
    email: String,
}

async fn sync_file(shell: &Shell) -> Result<()> {
    // Get the access token from defaults
    let token = Command::new("defaults")
        .arg("read")
        .arg("com.mschrage.fig")
        .arg("access_token")
        .output()
        .with_context(|| "Could not read access_token")?;

    let email = Credentials::load_credentials()
        .map(|creds| creds.email)
        .or_else(|_| {
            let out = Command::new("defaults")
                .arg("read")
                .arg("com.mschrage.fig")
                .arg("userEmail")
                .output()?;

            let email = String::from_utf8(out.stdout)?;

            anyhow::Ok(Some(email))
        })?;

    // Constuct the request body
    let body = serde_json::to_string(&DotfilesSourceRequest {
        email: email.unwrap_or_else(|| "".to_string()),
    })?;

    let download = reqwest::Client::new()
        .get(shell.get_remote_source()?)
        .header(
            "Authorization",
            format!("Bearer {}", String::from_utf8_lossy(&token.stdout).trim()),
        )
        .body(body)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    // Create path to dotfiles
    let cache_file = shell
        .get_data_path()
        .context("Could not get cache file path")?;
    let cache_folder = cache_file.parent().unwrap();

    // Create cache folder if it doesn't exist
    if !cache_folder.exists() {
        std::fs::create_dir_all(cache_folder)?;
    }

    let mut dest_file = std::fs::File::create(cache_file)?;
    dest_file.write_all(download.as_bytes())?;

    Ok(())
}

pub async fn sync_all_files() -> Result<()> {
    try_join!(
        sync_file(&Shell::Bash),
        sync_file(&Shell::Zsh),
        sync_file(&Shell::Fish),
    )?;

    Ok(())
}

/// Download the lastest dotfiles
pub async fn sync_cli() -> Result<()> {
    sync_all_files().await?;

    println!("Dotfiles synced!");

    Ok(())
}
