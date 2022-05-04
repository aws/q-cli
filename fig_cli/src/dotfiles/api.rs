use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{
    Context,
    Result,
};
use fig_auth::get_token;
use fig_settings::api_host;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use tracing::{
    debug,
    info,
};

use crate::plugins::api::PluginData;
use crate::util::shell::Shell;

/// The data for all the shells
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DotfilesData {
    #[serde(with = "time::serde::rfc3339::option")]
    pub updated_at: Option<time::OffsetDateTime>,
    pub plugins: Vec<PluginData>,
    #[serde(flatten)]
    pub dotfiles: HashMap<Shell, String>,
}

/// The data for a single shell
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DotfileData {
    pub dotfile: String,
    #[serde(with = "time::serde::rfc3339::option")]
    pub updated_at: Option<time::OffsetDateTime>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateStatus {
    New,
    Updated,
    NotUpdated,
}

pub async fn download_dotfiles() -> Result<UpdateStatus> {
    // Get the token
    let token = get_token().await?;

    let device_uniqueid = crate::util::get_machine_id();
    let plugins_directry = crate::plugins::download::plugin_data_dir().map(|p| p.to_string_lossy().to_string());

    let url: reqwest::Url = format!("{}/dotfiles/source/all", api_host()).parse()?;

    let debug_dotfiles = match fig_settings::state::get_value("developer.dotfiles.debug") {
        Ok(Some(serde_json::Value::Bool(true))) => Some("true"),
        _ => None,
    };

    let download = reqwest::Client::new()
        .get(url)
        .bearer_auth(token)
        .query(&[
            ("os", Some(std::env::consts::OS)),
            ("device", device_uniqueid.as_deref()),
            ("debug", debug_dotfiles),
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

    let all_json_path = all_file_path().context("Could not get cache file path")?;

    // Create dotfiles folder if it doesn't exist
    let dotfiles_folder = all_json_path.parent().unwrap();
    if !dotfiles_folder.exists() {
        std::fs::create_dir_all(dotfiles_folder)?;
    }

    // Write to all.json
    std::fs::write(all_json_path, download)?;

    // Write to the individual shell files
    for (shell, dotfile) in dotfiles.dotfiles {
        let shell_json_path = shell.get_data_path().context("Could not get cache file path")?;

        let dotfiles = DotfileData {
            dotfile,
            updated_at: dotfiles.updated_at,
        };

        std::fs::write(shell_json_path, &serde_json::to_vec(&dotfiles)?)?;
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
        (None, Some(_)) => Ok(UpdateStatus::New),
        (Some(previous_updated), Some(current_updated)) if current_updated > previous_updated => {
            Ok(UpdateStatus::Updated)
        },
        (_, _) => {
            info!("All dotfiles are up to date");
            Ok(UpdateStatus::NotUpdated)
        },
    }
}

#[must_use]
pub fn all_file_path() -> Option<PathBuf> {
    fig_directories::fig_data_dir().map(|dir| dir.join("shell").join("all.json"))
}
