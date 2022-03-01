//! Sync of dotfiles

use crate::util::shell::Shell;

use anyhow::{Context, Result};
use fig_auth::{get_email, get_token};
use serde::{Deserialize, Serialize};
use tokio::try_join;
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DotfilesSourceRequest {
    email: String,
}

// { name: "dotfiles", lastUpdate: 123456789 }

async fn sync_file(shell: &Shell, sync_when: SyncWhen) -> Result<()> {
    // Get the token
    let token = get_token().await?;
    let email = get_email();

    // OS macos, linux, windows
    // DEVICE, uniqueid
    // PLUGIN DIR, path

    // Constuct the request body
    let body = serde_json::to_string(&DotfilesSourceRequest {
        email: email.unwrap_or_default(),
    })?;

    let download = reqwest::Client::new()
        .get(shell.get_remote_source()?)
        .bearer_auth(token)
        .body(body)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    // Create path to dotfiles
    let mut cache_file = shell
        .get_data_path()
        .context("Could not get cache file path")?;

    // Append suffix to path if it should be synced later
    if sync_when == SyncWhen::Later {
        cache_file.set_extension("new");
    }

    let cache_folder = cache_file.parent().unwrap();

    // Create cache folder if it doesn't exist
    if !cache_folder.exists() {
        std::fs::create_dir_all(cache_folder)?;
    }

    std::fs::write(cache_file, download)?;

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncWhen {
    /// Sync the dotfiles immediately
    Immediately,
    /// Save to a temporary file and sync later
    Later,
}

pub async fn sync_all_shells(sync_when: SyncWhen) -> Result<()> {
    try_join!(
        sync_file(&Shell::Bash, sync_when),
        sync_file(&Shell::Zsh, sync_when),
        sync_file(&Shell::Fish, sync_when),
    )?;

    Ok(())
}

pub async fn sync_based_on_settings() -> Result<()> {
    let sync_when = match fig_settings::settings::get_value("dotfiles.syncImmediately") {
        Ok(Some(serde_json::Value::Bool(false))) => SyncWhen::Later,
        Ok(_) => SyncWhen::Immediately,
        Err(err) => {
            error!("Could not get dotfiles.syncImmediately: {}", err);
            SyncWhen::Immediately
        }
    };

    match sync_all_shells(sync_when).await {
        Ok(()) => match sync_when {
            SyncWhen::Immediately => {
                notify_terminals()?;
                info!("Dotfiles updated");
            }
            SyncWhen::Later => {
                info!("New dotfiles available");
            }
        },
        Err(err) => {
            error!("Could not sync dotfiles: {:?}", err);
        }
    };

    Ok(())
}

/// Notify dotfiles updates
pub fn notify_terminals() -> Result<()> {
    let tempdir = std::env::temp_dir();
    let dotfiles_updates_folder = tempdir.join("fig").join("dotfiles_updates");

    // Write true to all files in the dotfiles_updates folder
    if dotfiles_updates_folder.exists() {
        for file in dotfiles_updates_folder.read_dir()? {
            let file = file?;

            std::fs::write(file.path(), "true")?;
        }
    }

    Ok(())
}

/// Download the lastest dotfiles
pub async fn source_cli() -> Result<()> {
    sync_all_shells(SyncWhen::Immediately).await?;
    notify_terminals()?;
    Ok(())
}
