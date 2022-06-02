use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;

use clap::ArgEnum;
use serde::{
    Deserialize,
    Serialize,
};

use crate::process_info::get_parent_process_exe;

/// Shells supported by Fig
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ArgEnum)]
#[serde(rename_all = "camelCase")]
pub enum Shell {
    /// Bash shell
    Bash,
    /// Zsh shell
    Zsh,
    /// Fish shell
    Fish,
}

impl Display for Shell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Shell::Bash => f.write_str("bash"),
            Shell::Zsh => f.write_str("zsh"),
            Shell::Fish => f.write_str("fish"),
        }
    }
}

impl FromStr for Shell {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "bash" => Ok(Shell::Bash),
            "zsh" => Ok(Shell::Zsh),
            "fish" => Ok(Shell::Fish),
            _ => Err(()),
        }
    }
}

impl Shell {
    pub fn all() -> &'static [Self] {
        &[Shell::Bash, Shell::Zsh, Shell::Fish]
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
        } else {
            None
        }
    }

    /// Get the directory for the shell that contains the dotfiles
    pub fn get_config_directory(&self) -> Option<PathBuf> {
        match self {
            Shell::Bash => fig_directories::home_dir(),
            Shell::Zsh => std::env::var_os("ZDOTDIR")
                .or_else(|| std::env::var_os("FIG_ZDOTDIR"))
                .map(PathBuf::from)
                .or_else(fig_directories::home_dir),
            Shell::Fish => std::env::var_os("__fish_config_dir")
                .map(PathBuf::from)
                .or_else(|| fig_directories::home_dir().map(|home| home.join(".config").join("fish"))),
        }
    }

    pub fn get_data_path(&self) -> Option<PathBuf> {
        fig_directories::fig_data_dir().map(|dir| dir.join("shell").join(format!("{}.json", self)))
    }
}
