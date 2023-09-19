use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;

use fig_api_client::plugins::PluginData;
use fig_util::system_info::get_system_id;
use fig_util::{
    directories,
    Shell,
};
#[cfg(any(target_os = "macos", target_os = "linux"))]
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

use super::DotfilesError;

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
            let version_regex = regex::Regex::new(r"ProductVersion:\s*(\S+)").unwrap();
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

pub async fn download_dotfiles() -> Result<UpdateStatus, DotfilesError> {
    let device_uniqueid = get_system_id();
    let plugins_directory = directories::plugins_dir().map(|p| p.to_string_lossy().to_string()).ok();

    let debug_dotfiles = match fig_settings::state::get_value("developer.dotfiles.debug") {
        Ok(Some(serde_json::Value::Bool(true))) => Some("true"),
        _ => None,
    };

    let download = fig_request::Request::get("/dotfiles/source/all")
        .auth()
        .query(&[
            ("os", Some(std::env::consts::OS)),
            ("architecture", Some(std::env::consts::ARCH)),
            ("device", device_uniqueid),
            ("debug", debug_dotfiles),
            ("pluginsDirectory", plugins_directory.as_deref()),
            #[cfg(target_os = "linux")]
            ("linuxKernelVersion", LINUX_KERNEL_VERSION.as_deref()),
            #[cfg(target_os = "macos")]
            ("macosVersion", MACOS_VERSION.as_deref()),
        ])
        .text()
        .await?;

    // Parse the JSON
    let dotfiles: DotfilesData = serde_json::from_str(&download)?;

    let all_json_path = all_file_path()?;

    // Create dotfiles folder if it doesn't exist
    let dotfiles_folder = all_json_path.parent().unwrap();
    if !dotfiles_folder.exists() {
        std::fs::create_dir_all(dotfiles_folder)?;
    }

    // Write to all.json
    let mut file_opts = std::fs::OpenOptions::new();
    file_opts.write(true).create(true).truncate(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        file_opts.mode(0o600);
    }

    let mut file = file_opts.open(&all_json_path)?;
    file.write_all(download.as_bytes())?;

    // Write to the individual shell files
    for (shell, dotfile) in dotfiles.dotfiles {
        let shell_json_path = shell.get_data_path()?;

        let dotfiles = DotfileData {
            dotfile,
            updated_at: dotfiles.updated_at,
        };

        let mut file_opts = std::fs::OpenOptions::new();
        file_opts.write(true).create(true).truncate(true);

        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            file_opts.mode(0o600);
        }

        let mut file = file_opts.open(shell_json_path)?;
        file.write_all(&serde_json::to_vec(&dotfiles)?)?;
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

pub fn all_file_path() -> Result<PathBuf, fig_util::directories::DirectoryError> {
    Ok(directories::fig_data_dir()?.join("shell").join("all.json"))
}
