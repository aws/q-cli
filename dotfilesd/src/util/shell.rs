use std::{
    fmt::Display,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    str::FromStr,
};

use anyhow::{Context, Result};
use clap::ArgEnum;
use regex::Regex;
use reqwest::Url;
use serde::{Deserialize, Serialize};

use super::{fig_dir, project_dir};

#[derive(Debug, Copy, Clone, PartialEq, Eq, ArgEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum When {
    Pre,
    Post,
}

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

pub struct ShellIntegration {
    pub shell: Shell,
    pub when: When,
}

impl ShellIntegration {
    fn description(&self) -> String {
        format!("# {:?} dotfiles eval", self.when)
    }

    fn source_text(&self) -> String {
        let when_text = match self.when {
            When::Pre => "pre",
            When::Post => "post",
        };
        match self.shell {
            Shell::Fish => format!("eval (dotfiles init {} {})", self.shell, when_text),
            _ => format!("eval \"$(dotfiles init {} {})\"", self.shell, when_text),
        }
    }

    pub fn text(&self) -> String {
        format!("{}\n{}\n", self.description(), self.source_text())
    }

    pub fn get_source_regex(&self) -> Result<Regex> {
        let r = format!(
            r#"(?:{}\n)?{}\n{{0,2}}"#,
            regex::escape(&self.description()),
            regex::escape(&self.source_text()),
        );
        Regex::new(&r).context("Invalid source regex")
    }
}

#[derive(Debug, Clone)]
pub struct ShellFileIntegration {
    pub shell: Shell,
    pub path: PathBuf,
    pub pre: bool,
    pub post: bool,
    pub remove_on_uninstall: bool,
}

impl ShellFileIntegration {
    pub fn pre_integration(&self) -> Option<ShellIntegration> {
        if self.pre {
            Some(ShellIntegration {
                when: When::Pre,
                shell: self.shell,
            })
        } else {
            None
        }
    }

    pub fn post_integration(&self) -> Option<ShellIntegration> {
        if self.post {
            Some(ShellIntegration {
                when: When::Post,
                shell: self.shell,
            })
        } else {
            None
        }
    }

    pub fn uninstall(&self) -> Result<()> {
        if self.path.exists() {
            if self.remove_on_uninstall {
                std::fs::remove_file(self.path.as_path())?;
                return Ok(());
            }

            let mut file = File::open(&self.path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            if let Some(pre) = self.pre_integration() {
                contents = pre.get_source_regex()?.replace_all(&contents, "").into();
            }
            if let Some(post) = self.post_integration() {
                contents = post.get_source_regex()?.replace_all(&contents, "").into();
            }

            let mut file = File::create(&self.path)?;
            file.write_all(contents.as_bytes())?;
        }

        Ok(())
    }

    pub fn install(&self) -> Result<()> {
        let mut contents = String::new();
        if self.path.exists() {
            let mut file = File::open(&self.path)?;
            file.read_to_string(&mut contents)?;
        }

        let mut modified = false;
        let mut new_contents = String::new();

        if let Some(integration) = self.pre_integration() {
            if !integration.get_source_regex()?.is_match(&contents) {
                new_contents.push_str(&integration.text());
                new_contents.push('\n');
                modified = true;
            }
        }

        new_contents.push_str(&contents);

        if let Some(integration) = self.post_integration() {
            if !integration.get_source_regex()?.is_match(&contents) {
                new_contents.push('\n');
                new_contents.push_str(&integration.text());
                modified = true;
            }
        }

        if modified {
            let mut file = File::create(&self.path)?;
            file.write_all(new_contents.as_bytes())?;
        }

        Ok(())
    }
}

impl Shell {
    pub fn all() -> &'static [Self] {
        &[Shell::Bash, Shell::Zsh, Shell::Fish]
    }

    pub fn get_shell_integrations(&self) -> Result<Vec<ShellFileIntegration>> {
        let base_dir = directories::BaseDirs::new().context("Failed to get base directories")?;
        let home_dir = base_dir.home_dir();

        let path = match self {
            Shell::Bash => {
                let mut configs = vec![home_dir.join(".bashrc")];
                let other_configs: Vec<_> = [".profile", ".bash_login", ".bash_profile"]
                    .into_iter()
                    .map(|f| home_dir.join(f))
                    .collect();
                configs.extend(other_configs.clone().into_iter().filter(|f| f.exists()));
                // Include .profile if none of [.profile, .bash_login, .bash_profile] exist.
                if configs.len() == 1 {
                    configs.push(other_configs.first().unwrap().into());
                }
                configs
                    .into_iter()
                    .map(|path| ShellFileIntegration {
                        path,
                        pre: true,
                        post: true,
                        shell: *self,
                        remove_on_uninstall: false,
                    })
                    .collect()
            }
            Shell::Zsh => {
                let zdotdir = std::env::var("ZDOTDIR")
                    .or_else(|_| std::env::var("FIG_ZDOTDIR"))
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| home_dir.into());
                vec![zdotdir.join(".zshrc"), zdotdir.join(".zprofile")]
                    .into_iter()
                    .map(|path| ShellFileIntegration {
                        path,
                        pre: true,
                        post: true,
                        shell: *self,
                        remove_on_uninstall: false,
                    })
                    .collect()
            }
            Shell::Fish => {
                let fish_config_dir = std::env::var("__fish_config_dir")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| home_dir.join(".config").join("fish"))
                    .join("conf.d");
                vec![
                    ShellFileIntegration {
                        path: fish_config_dir.join("00_fig_pre.fish"),
                        shell: *self,
                        pre: true,
                        post: false,
                        remove_on_uninstall: true,
                    },
                    ShellFileIntegration {
                        path: fish_config_dir.join("99_fig_post.fish"),
                        shell: *self,
                        pre: false,
                        post: true,
                        remove_on_uninstall: true,
                    },
                ]
            }
        };

        Ok(path)
    }

    pub fn get_fig_integration_path(&self, when: &When) -> Option<PathBuf> {
        let fig = fig_dir()?;
        let path = match (self, when) {
            (Shell::Fish, When::Pre) => fig.join("shell").join("pre.fish"),
            (Shell::Fish, When::Post) => fig.join("shell").join("post.fish"),
            (_, When::Pre) => fig.join("shell").join("pre.sh"),
            (_, When::Post) => fig.join("fig.sh"),
        };
        Some(path)
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
