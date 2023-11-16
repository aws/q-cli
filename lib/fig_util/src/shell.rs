use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;

use clap::ValueEnum;
use regex::Regex;
use serde::{
    Deserialize,
    Serialize,
};

use crate::process_info::get_parent_process_exe;
use crate::{
    directories,
    Error,
};

/// Shells supported by Fig
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "camelCase")]
pub enum Shell {
    /// Bash shell
    Bash,
    /// Zsh shell
    Zsh,
    /// Fish shell
    Fish,
    /// Nu shell
    Nu,
}

impl Display for Shell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
            Shell::Nu => "nu",
        })
    }
}

impl FromStr for Shell {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "bash" => Ok(Shell::Bash),
            "zsh" => Ok(Shell::Zsh),
            "fish" => Ok(Shell::Fish),
            "nu" => Ok(Shell::Nu),
            _ => Err(()),
        }
    }
}

impl Shell {
    pub fn all() -> &'static [Self] {
        &[Shell::Bash, Shell::Zsh, Shell::Fish, Shell::Nu]
    }

    pub fn current_shell() -> Option<Self> {
        let parent_exe = get_parent_process_exe()?;
        let parent_exe_name = parent_exe.to_str()?;
        if parent_exe_name.contains("bash") {
            Some(Shell::Bash)
        } else if parent_exe_name.contains("zsh") {
            Some(Shell::Zsh)
        } else if parent_exe_name.contains("fish") {
            Some(Shell::Fish)
        } else if parent_exe_name == "nu" || parent_exe_name == "nushell" {
            Some(Shell::Nu)
        } else {
            None
        }
    }

    pub async fn current_shell_version() -> Result<(Self, String), Error> {
        let parent_exe = get_parent_process_exe().ok_or(Error::NoParentProcess)?;
        let parent_exe_name = parent_exe
            .to_str()
            .ok_or_else(|| Error::UnknownShell(parent_exe.to_string_lossy().into()))?;
        if parent_exe_name.contains("bash") {
            let version_output = tokio::process::Command::new(parent_exe)
                .arg("--version")
                .output()
                .await?;

            let re = Regex::new(r"GNU bash, version (\d+\.\d+\.\d+)").unwrap();
            let version_capture = re.captures(std::str::from_utf8(&version_output.stdout)?);
            let version = version_capture.unwrap().get(1).unwrap().as_str();
            Ok((Shell::Bash, version.into()))
        } else if parent_exe_name.contains("zsh") {
            let version_output = tokio::process::Command::new(parent_exe)
                .arg("--version")
                .output()
                .await?;

            let re = Regex::new(r"(\d+\.\d+)").unwrap();
            let version_capture = re.captures(std::str::from_utf8(&version_output.stdout)?);
            let version = version_capture.unwrap().get(1).unwrap().as_str();
            Ok((Shell::Zsh, format!("{version}.0")))
        } else if parent_exe_name.contains("fish") {
            let version_output = tokio::process::Command::new(parent_exe)
                .arg("--version")
                .output()
                .await?;

            let re = Regex::new(r"(\d+\.\d+\.\d+)").unwrap();
            let version_capture = re.captures(std::str::from_utf8(&version_output.stdout)?);
            let version = version_capture.unwrap().get(1).unwrap().as_str();
            Ok((Shell::Fish, version.into()))
        } else if parent_exe_name == "nu" || parent_exe_name == "nushell" {
            let version_output = tokio::process::Command::new(parent_exe)
                .arg("--version")
                .output()
                .await?;
            let version = std::str::from_utf8(&version_output.stdout)?.trim();
            Ok((Shell::Nu, version.into()))
        } else {
            Err(Error::UnknownShell(parent_exe_name.into()))
        }
    }

    /// Get the directory for the shell that contains the dotfiles
    pub fn get_config_directory(&self) -> Result<PathBuf, directories::DirectoryError> {
        match self {
            Shell::Bash => Ok(directories::home_dir()?),
            Shell::Zsh => match std::env::var_os("ZDOTDIR")
                .or_else(|| std::env::var_os("CW_ZDOTDIR"))
                .map(PathBuf::from)
            {
                Some(dir) => Ok(dir),
                None => Ok(directories::home_dir()?),
            },
            Shell::Fish => match std::env::var_os("__fish_config_dir").map(PathBuf::from) {
                Some(dir) => Ok(dir),
                None => Ok(directories::home_dir()?.join(".config").join("fish")),
            },
            Shell::Nu => Ok(directories::config_dir()?.join("nushell")),
        }
    }

    pub fn get_data_path(&self) -> Result<PathBuf, directories::DirectoryError> {
        Ok(directories::fig_data_dir()?.join("shell").join(format!("{self}.json")))
    }
}
