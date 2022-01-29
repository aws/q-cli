use std::{env, fmt::Display, path::PathBuf, str::FromStr};

use anyhow::{Context, Result};
use clap::ArgEnum;
use reqwest::Url;
use serde::{Deserialize, Serialize};

use super::project_dir;

/// Shells supported by Fig
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ArgEnum)]
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

impl FromStr for Shell {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "bash" => Ok(Shell::Bash),
            "zsh" => Ok(Shell::Zsh),
            "fish" => Ok(Shell::Fish),
            _ => Err(anyhow::anyhow!("Unknown shell: {}", s)),
        }
    }
}

impl Shell {
    pub fn all() -> &'static [Self] {
        &[Shell::Bash, Shell::Zsh, Shell::Fish]
    }

    pub fn get_config_path(&self) -> Result<PathBuf> {
        let base_dir = directories::BaseDirs::new().context("Failed to get base directories")?;
        let home_dir = base_dir.home_dir();

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

    pub fn get_data_path(&self) -> Option<PathBuf> {
        Some(
            project_dir()?
                .data_local_dir()
                .join("dotfiles")
                .join(format!("{}.json", self)),
        )
    }

    pub fn get_remote_source(&self) -> Result<Url> {
        Ok(format!("https://api.fig.io/dotfiles/source/{}", self).parse()?)
    }
}
