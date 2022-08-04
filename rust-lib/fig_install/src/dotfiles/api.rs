use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{
    Context,
    Result,
};
use fig_util::{
    directories,
    Shell,
};
#[cfg(unix)]
use once_cell::sync::Lazy;
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

#[cfg(target_os = "linux")]
static LINUX_KERNEL_VERSION: Lazy<Option<String>> = Lazy::new(|| {
    use std::process::Command;

    Command::new("uname")
        .arg("-r")
        .output()
        .ok()
        .and_then(|output| std::str::from_utf8(&output.stdout).ok().map(|s| s.trim().to_owned()))
});

#[cfg(target_os = "macos")]
static MACOS_VERSION: Lazy<Option<String>> = Lazy::new(|| {
    use std::process::Command;

    Command::new("sw_vers")
        .output()
        .ok()
        .and_then(|output| -> Option<String> {
            let version_info = std::str::from_utf8(&output.stdout).ok().map(|s| s.trim().to_owned())?;
            let version_regex = regex::Regex::new(r#"ProductVersion:\s*(\S+)"#).unwrap();
            let version = version_regex
                .captures(&version_info)
                .and_then(|c| c.get(1))
                .map(|v| v.as_str().into());
            version
        })
});

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
    let device_uniqueid = fig_util::get_system_id().ok();
    let plugins_directory = crate::plugins::plugin_data_dir().map(|p| p.to_string_lossy().to_string());

    let debug_dotfiles = match fig_settings::state::get_value("developer.dotfiles.debug") {
        Ok(Some(serde_json::Value::Bool(true))) => Some("true"),
        _ => None,
    };

    let download = fig_request::Request::get("/dotfiles/source/all")
        .auth()
        .query(&[
            ("os", Some(std::env::consts::OS)),
            ("architecture", Some(std::env::consts::ARCH)),
            ("device", device_uniqueid.as_deref()),
            ("debug", debug_dotfiles),
            ("pluginsDirectory", plugins_directory.ok().as_deref()),
            #[cfg(target_os = "linux")]
            ("linuxKernelVersion", LINUX_KERNEL_VERSION.as_deref()),
            #[cfg(target_os = "macos")]
            ("macosVersion", MACOS_VERSION.as_deref()),
        ])
        .text()
        .await?;

    // Parse the JSON
    let dotfiles: DotfilesData = serde_json::from_str(&download).context("Failed to parse JSON")?;
    debug!("dotfiles: {:?}", dotfiles.dotfiles);

    let all_json_path = all_file_path()?;

    // Create dotfiles folder if it doesn't exist
    let dotfiles_folder = all_json_path.parent().unwrap();
    if !dotfiles_folder.exists() {
        std::fs::create_dir_all(dotfiles_folder)?;
    }

    // Write to all.json
    std::fs::write(all_json_path, download)?;

    // Write to the individual shell files
    for (shell, dotfile) in dotfiles.dotfiles {
        let shell_json_path = shell.get_data_path()?;

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

pub fn all_file_path() -> Result<PathBuf> {
    Ok(directories::fig_data_dir()?.join("shell").join("all.json"))
}
