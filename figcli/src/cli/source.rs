//! Sync of dotfiles

use crate::util::shell::Shell;

use anyhow::{Context, Result};
use fig_auth::get_token;
use serde::{Deserialize, Serialize};
use serde_json::json;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use tokio::try_join;
use tracing::{debug, error, info};

use super::init::DotfileData;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DotfilesSourceRequest {
    email: String,
}

// { name: "dotfiles", lastUpdate: 123456789 }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateStatus {
    Updated,
    NotUpdated,
}

async fn sync_file(shell: &Shell, sync_when: SyncWhen) -> Result<UpdateStatus> {
    // Get the token
    let token = get_token().await?;

    let device_uniqueid = crate::util::get_machine_id();
    let plugins_directry =
        crate::plugins::download::plugin_data_dir().map(|p| p.to_string_lossy().to_string());

    let download = reqwest::Client::new()
        .get(shell.get_remote_source()?)
        .bearer_auth(token)
        .query(&[
            ("os", Some(std::env::consts::OS)),
            ("device", device_uniqueid.as_deref()),
            ("pluginsDirectory", plugins_directry.as_deref()),
        ])
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    // Parse the JSON
    let dotfiles: DotfileData = serde_json::from_str(&download).context("Failed to parse JSON")?;

    let last_updated =
        fig_settings::state::get_value(format!("dotfiles.{}.{}", shell, "lastUpdate"))?
            .and_then(|v| v.as_str().map(String::from))
            .and_then(|s| OffsetDateTime::parse(&s, &Rfc3339).ok());

    debug!("dotfiles_json: {:?}", dotfiles.dotfile);
    debug!(
        "dotfiles_last_updated: {:?}",
        dotfiles.updated_at.map(|t| t.unix_timestamp_nanos())
    );
    debug!(
        "last_updated: {:?}",
        last_updated.map(|t| t.unix_timestamp_nanos())
    );

    let update_dotfiles = || {
        // Create path to dotfiles
        let mut json_file = shell
            .get_data_path()
            .context("Could not get cache file path")?;

        // Append suffix to path if it should be synced later
        if sync_when == SyncWhen::Later {
            json_file.set_extension("new");
        }

        let dotfiles_folder = json_file.parent().unwrap();

        // Create dotfiles folder if it doesn't exist
        if !dotfiles_folder.exists() {
            std::fs::create_dir_all(dotfiles_folder)?;
        }

        std::fs::write(json_file, download)?;

        fig_settings::state::set_value(
            format!("dotfiles.{}.lastUpdated", shell),
            json!(dotfiles.updated_at.and_then(|t| t.format(&Rfc3339).ok())),
        )?;

        anyhow::Ok(())
    };

    match (last_updated, dotfiles.updated_at) {
        (Some(previous_updated), Some(current_updated)) if current_updated > previous_updated => {
            update_dotfiles()?;
            Ok(UpdateStatus::Updated)
        }
        (None, Some(_)) => {
            update_dotfiles()?;
            Ok(UpdateStatus::Updated)
        }
        (_, _) => {
            info!("{} dotfiles are up to date", shell);
            Ok(UpdateStatus::NotUpdated)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncWhen {
    /// Sync the dotfiles immediately
    Immediately,
    /// Save to a temporary file and sync later
    Later,
}

pub async fn sync_all_shells(sync_when: SyncWhen) -> Result<UpdateStatus> {
    let (bash_updated, zsh_updated, fish_updated) = try_join!(
        sync_file(&Shell::Bash, sync_when),
        sync_file(&Shell::Zsh, sync_when),
        sync_file(&Shell::Fish, sync_when),
    )?;

    if bash_updated == UpdateStatus::Updated
        || zsh_updated == UpdateStatus::Updated
        || fish_updated == UpdateStatus::Updated
    {
        Ok(UpdateStatus::Updated)
    } else {
        Ok(UpdateStatus::NotUpdated)
    }
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
        Ok(update_status) => match (sync_when, update_status) {
            (SyncWhen::Immediately, UpdateStatus::Updated) => {
                notify_all_terminals(TerminalNotification::NewUpdates)?;
                info!("Dotfiles updated");
            }
            (SyncWhen::Later, UpdateStatus::Updated) => {
                info!("New dotfiles available");
            }
            (_, UpdateStatus::NotUpdated) => {
                info!("Dotfiles are up to date");
            }
        },
        Err(err) => {
            error!("Could not sync dotfiles: {:?}", err);
        }
    };

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalNotification {
    NewUpdates,
    Source,
}

impl std::str::FromStr for TerminalNotification {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "newUpdates" => Ok(TerminalNotification::NewUpdates),
            "source" => Ok(TerminalNotification::Source),
            _ => Err(anyhow::anyhow!("Invalid terminal notification: {}", s)),
        }
    }
}

impl std::fmt::Display for TerminalNotification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TerminalNotification::NewUpdates => write!(f, "newUpdates"),
            TerminalNotification::Source => write!(f, "source"),
        }
    }
}

pub fn notify_terminal(
    session_id: impl AsRef<str>,
    notification: TerminalNotification,
) -> Result<()> {
    let dotfiles_update_path = std::env::temp_dir()
        .join("fig")
        .join("dotfiles_updates")
        .join(session_id.as_ref());

    std::fs::write(dotfiles_update_path, notification.to_string())?;

    Ok(())
}

/// Notify dotfiles updates
pub fn notify_all_terminals(notification: TerminalNotification) -> Result<()> {
    let tempdir = std::env::temp_dir();
    let dotfiles_updates_folder = tempdir.join("fig").join("dotfiles_updates");

    // Write true to all files in the dotfiles_updates folder
    if dotfiles_updates_folder.exists() {
        for file in dotfiles_updates_folder.read_dir()? {
            let file = file?;

            std::fs::write(file.path(), notification.to_string())?;
        }
    }

    Ok(())
}

/// Download the lastest dotfiles
pub async fn source_cli() -> Result<()> {
    sync_all_shells(SyncWhen::Immediately).await?;
    if let Ok(session_id) = std::env::var("TERM_SESSION_ID") {
        notify_terminal(session_id, TerminalNotification::Source)?;
    }

    Ok(())
}
