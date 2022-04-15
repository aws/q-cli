use anyhow::{Context, Result};
use clap::ArgEnum;
use fig_settings::api_host;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf, str::FromStr};

use crate::integrations::shell::{
    DotfileShellIntegration, ShellIntegration, ShellScriptShellIntegration, When,
};
use crate::util::get_parent_process_exe;

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

    pub fn current_shell() -> Option<Self> {
        let parent_exe = get_parent_process_exe().ok()?;
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

    pub fn get_shell_integrations(&self) -> Result<Vec<Box<dyn ShellIntegration>>> {
        let home_dir = fig_directories::home_dir().context("Failed to get base directories")?;

        let integrations: Vec<Box<dyn ShellIntegration>> = match self {
            Shell::Bash => {
                let mut configs = vec![".bashrc"];
                let other_configs: Vec<_> = vec![".profile", ".bash_login", ".bash_profile"];
                configs.extend(
                    other_configs
                        .clone()
                        .into_iter()
                        .filter(|f| home_dir.join(f).exists()),
                );
                // Include .profile if none of [.profile, .bash_login, .bash_profile] exist.
                if configs.len() == 1 {
                    configs.push(other_configs.first().unwrap());
                }
                configs
                    .into_iter()
                    .map(|filename| {
                        Box::new(DotfileShellIntegration {
                            pre: true,
                            post: true,
                            shell: *self,
                            dotfile_directory: home_dir.clone(),
                            dotfile_name: filename,
                        }) as Box<dyn ShellIntegration>
                    })
                    .collect()
            }
            Shell::Zsh => {
                let zdotdir = std::env::var("ZDOTDIR")
                    .or_else(|_| std::env::var("FIG_ZDOTDIR"))
                    .map_or_else(|_| home_dir, PathBuf::from);
                vec![".zshrc", ".zprofile"]
                    .into_iter()
                    .map(|filename| {
                        Box::new(DotfileShellIntegration {
                            pre: true,
                            post: true,
                            shell: *self,
                            dotfile_directory: zdotdir.clone(),
                            dotfile_name: filename,
                        }) as Box<dyn ShellIntegration>
                    })
                    .collect()
            }
            Shell::Fish => {
                let fish_config_dir = std::env::var("__fish_config_dir")
                    .map_or_else(|_| home_dir.join(".config").join("fish"), PathBuf::from)
                    .join("conf.d");
                vec![
                    Box::new(ShellScriptShellIntegration {
                        when: When::Pre,
                        shell: *self,
                        path: fish_config_dir.join("00_fig_pre.fish"),
                    }),
                    Box::new(ShellScriptShellIntegration {
                        when: When::Post,
                        shell: *self,
                        path: fish_config_dir.join("99_fig_post.fish"),
                    }),
                ]
            }
        };

        Ok(integrations)
    }

    pub fn get_fig_integration_source(&self, when: &When) -> &'static str {
        match (self, when) {
            (Shell::Fish, When::Pre) => include_str!("../integrations/shell/scripts/pre.fish"),
            (Shell::Fish, When::Post) => include_str!("../integrations/shell/scripts/post.fish"),
            (Shell::Zsh, When::Pre) => include_str!("../integrations/shell/scripts/pre.sh"),
            (Shell::Zsh, When::Post) => include_str!("../integrations/shell/scripts/post.zsh"),
            (Shell::Bash, When::Pre) => {
                concat!(
                    "function __fig_source_bash_preexec() {\n",
                    include_str!("../integrations/shell/scripts/bash-preexec.sh"),
                    "\n}\n\
                    __fig_source_bash_preexec\n\
                    function __bp_adjust_histcontrol() { :; }\n",
                    include_str!("../integrations/shell/scripts/pre.sh")
                )
            }
            (Shell::Bash, When::Post) => {
                concat!(
                    "function __fig_source_bash_preexec() {\n",
                    include_str!("../integrations/shell/scripts/bash-preexec.sh"),
                    "\n}\n\
                    __fig_source_bash_preexec\n\
                    function __bp_adjust_histcontrol() { :; }\n",
                    include_str!("../integrations/shell/scripts/post.bash")
                )
            }
        }
    }

    #[must_use]
    pub fn get_data_path(&self) -> Option<PathBuf> {
        fig_directories::fig_data_dir().map(|dir| dir.join("shell").join(format!("{}.json", self)))
    }

    pub fn get_remote_source(&self) -> Result<Url> {
        Ok(format!("{}/dotfiles/source/{}", api_host(), self).parse()?)
    }
}
