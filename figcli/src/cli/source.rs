//! Sync of dotfiles

use crate::{
    cli::init::DotfilesData,
    util::{api::api_host, is_logged_in, shell::Shell},
};

use anyhow::{Context, Result};
use fig_auth::get_token;
use serde::{Deserialize, Serialize};
use serde_json::json;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
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

async fn _sync_file(shell: &Shell, sync_when: SyncWhen) -> Result<UpdateStatus> {
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
        fig_settings::state::get_value(format!("dotfiles.{}.{}", shell, "lastUpdated"))?
            .and_then(|v| v.as_str().map(String::from))
            .and_then(|s| OffsetDateTime::parse(&s, &Rfc3339).ok());

    debug!("dotfiles_json: {:?}", dotfiles.dotfile);
    debug!(
        "new lastUpdated: {:?}",
        dotfiles.updated_at.and_then(|t| t.format(&Rfc3339).ok())
    );
    debug!(
        "old lastUpdated: {:?}",
        last_updated.and_then(|t| t.format(&Rfc3339).ok())
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
            update_dotfiles()?;
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

pub async fn sync_all_shells() -> Result<UpdateStatus> {
    // Get the token
    let token = get_token().await?;

    let device_uniqueid = crate::util::get_machine_id();
    let plugins_directry =
        crate::plugins::download::plugin_data_dir().map(|p| p.to_string_lossy().to_string());

    let url: reqwest::Url = format!("{}/dotfiles/source/all", api_host()).parse()?;

    let download = reqwest::Client::new()
        .get(url)
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
    let dotfiles: DotfilesData = serde_json::from_str(&download).context("Failed to parse JSON")?;
    debug!("dotfiles: {:?}", dotfiles.dotfiles);

    // Create path to dotfiles
    let json_file = fig_directories::fig_data_dir()
        .map(|dir| dir.join("shell").join("all.json"))
        .context("Could not get cache file path")?;

    // Create dotfiles folder if it doesn't exist
    let dotfiles_folder = json_file.parent().unwrap();
    if !dotfiles_folder.exists() {
        std::fs::create_dir_all(dotfiles_folder)?;
    }

    // Write to all.json
    std::fs::write(json_file, download)?;

    // Write to the individual shell files
    for (shell, dotfile) in dotfiles.dotfiles.into_iter() {
        let json_file = shell
            .get_data_path()
            .context("Could not get cache file path")?;

        let dotfiles = DotfileData {
            dotfile,
            updated_at: dotfiles.updated_at,
        };

        std::fs::write(json_file, &serde_json::to_vec(&dotfiles)?)?;
    }

    // Set the last updated time
    let last_updated = fig_settings::state::get_value("dotfiles.all.lastUpdated")?
        .and_then(|v| v.as_str().map(String::from))
        .and_then(|s| OffsetDateTime::parse(&s, &Rfc3339).ok());

    debug!(
        "new lastUpdated: {:?}",
        dotfiles.updated_at.and_then(|t| t.format(&Rfc3339).ok())
    );
    debug!(
        "old lastUpdated: {:?}",
        last_updated.and_then(|t| t.format(&Rfc3339).ok())
    );

    fig_settings::state::set_value(
        "dotfiles.all.lastUpdated",
        json!(dotfiles.updated_at.and_then(|t| t.format(&Rfc3339).ok())),
    )?;

    // Return the status of if the update is newer than the last update
    match (last_updated, dotfiles.updated_at) {
        (Some(previous_updated), Some(current_updated)) if current_updated > previous_updated => {
            Ok(UpdateStatus::Updated)
        }
        (_, _) => {
            info!("All dotfiles are up to date");
            Ok(UpdateStatus::NotUpdated)
        }
    }
}

pub async fn sync_based_on_settings() -> Result<()> {
    // Guard if the user has disabled immediate syncing
    match fig_settings::settings::get_value("dotfiles.syncImmediately") {
        Ok(Some(serde_json::Value::Bool(false))) => {
            return Ok(());
        }
        Ok(_) => {}
        Err(err) => {
            error!("Could not get dotfiles.syncImmediately: {}", err);
        }
    };

    match sync_all_shells().await {
        Ok(UpdateStatus::Updated) => {
            notify_all_terminals(TerminalNotification::NewUpdates)?;
            info!("Dotfiles updated");
        }
        Ok(UpdateStatus::NotUpdated) => info!("Dotfiles are up to date"),
        Err(err) => error!("Could not sync dotfiles: {:?}", err),
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
    if !is_logged_in() {
        anyhow::bail!("Must be logged in to sync dotfiles");
    }
    sync_all_shells()
        .await
        .context("Could not sync remote dotfiles")?;
    if let Ok(session_id) = std::env::var("TERM_SESSION_ID") {
        notify_terminal(session_id, TerminalNotification::Source)?;
    }

    Ok(())
}
