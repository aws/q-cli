use std::{env, fmt::Display, path::PathBuf};

use anyhow::{Context, Result};
use clap::ArgEnum;
use reqwest::Url;
use serde::{Deserialize, Serialize};

/// Shells supported by Fig
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, ArgEnum)]
#[serde(rename_all = "kebab-case")]
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

impl Shell {
    pub fn get_config_path(&self) -> Result<PathBuf> {
        let home_dir = dirs::home_dir().context("Could not get home directory")?;

        let path = match self {
            Shell::Bash => home_dir.join(".bashrc"),
            Shell::Zsh => match env::var("ZDOTDIR")
                .or_else(|_| env::var("FIG_ZDOTDIR"))
                .map(PathBuf::from)
            {
                Ok(zdotdir) => {
                    let zdot_path = zdotdir.join(".zshrc");
                    if zdot_path.exists() {
                        zdot_path
                    } else {
                        home_dir.join(".zshrc")
                    }
                }
                Err(_) => home_dir.join(".zshrc"),
            },
            Shell::Fish => home_dir.join(".config/fish/config.fish"),
        };

        Ok(path)
    }

    pub fn get_cache_path(&self) -> Result<PathBuf> {
        Ok(dirs::cache_dir()
            .context("Could not get cache directory")?
            .join("fig")
            .join("dotfiles")
            .join(format!("{}.json", self)))
    }

    pub fn get_remote_source(&self) -> Result<Url> {
        Ok(format!("https://api.fig.io/dotfiles/source/{}", self).parse()?)
    }
}
