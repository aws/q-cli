use anyhow::{Context, Result};
use clap::ArgEnum;
use fig_settings::api_host;
use regex::Regex;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    str::FromStr,
};
use time::OffsetDateTime;

use crate::util::get_parent_process_exe;

#[derive(Debug, Copy, Clone, PartialEq, Eq, ArgEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum When {
    Pre,
    Post,
}

impl std::fmt::Display for When {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            When::Pre => write!(f, "pre"),
            When::Post => write!(f, "post"),
        }
    }
}

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

pub struct ShellIntegration {
    pub shell: Shell,
    pub when: When,
}

impl ShellIntegration {
    fn description(&self) -> String {
        match self.when {
            When::Pre => "# Fig pre block. Keep at the top of this file.".into(),
            When::Post => "# Fig post block. Keep at the bottom of this file.".into(),
        }
    }

    fn source_text(&self) -> String {
        let when_text = match self.when {
            When::Pre => "pre",
            When::Post => "post",
        };
        let eval_line = match self.shell {
            Shell::Fish => format!(
                "eval (fig init {} {} | string split0)",
                self.shell, when_text
            ),
            _ => format!("eval \"$(fig init {} {})\"", self.shell, when_text),
        };

        match self.when {
            When::Pre => match self.shell {
                Shell::Fish => format!("set -Ua fish_user_paths $HOME/.local/bin\n{}", eval_line),
                _ => format!(
                    "export PATH=\"${{PATH}}:${{HOME}}/.local/bin\"\n{}",
                    eval_line
                ),
            },
            When::Post => eval_line,
        }
    }

    pub fn text(&self) -> String {
        format!("{}\n{}\n", self.description(), self.source_text())
    }

    pub fn get_source_regex(&self, constrain_position: bool) -> Result<Regex> {
        let (prefix, suffix) = if constrain_position {
            match self.when {
                When::Pre => ("^", ""),
                When::Post => ("", "$"),
            }
        } else {
            ("", "")
        };
        let r = format!(
            r#"(?:{}\n)?(?:{}\n)?{}\n{{0,2}}{}"#,
            prefix,
            regex::escape(&self.description()),
            regex::escape(&self.source_text()),
            suffix
        );
        Regex::new(&r).context("Invalid source regex")
    }
}

#[derive(Debug, Clone)]
pub struct ShellFileIntegration {
    pub shell: Shell,
    pub directory: PathBuf,
    pub filename: &'static str,
    pub pre: bool,
    pub post: bool,
    pub remove_on_uninstall: bool,
}

impl std::fmt::Display for ShellFileIntegration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.filename)
    }
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

    pub fn path(&self) -> PathBuf {
        self.directory.join(self.filename)
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
        let path = self.path();
        if path.exists() {
            if self.remove_on_uninstall {
                std::fs::remove_file(path.as_path())?;
                return Ok(());
            }

            let mut contents = std::fs::read_to_string(&path)?;

            // Remove comment lines
            contents = Regex::new(r"(?mi)^#.*fig.*var.*$\n?")?
                .replace_all(&contents, "")
                .into();

            contents = Regex::new(
                r"(?mi)^#.*Please make sure this block is at the .* of this file.*$\n?",
            )?
            .replace_all(&contents, "")
            .into();

            // Remove old integration pre
            contents =
                Regex::new(r"\[ -s ~/\.fig/shell/pre\.sh \] && source ~/\.fig/shell/pre\.sh\n?")?
                    .replace_all(&contents, "")
                    .into();

            // Remove old integration post
            contents = Regex::new(r"\[ -s ~/\.fig/fig\.sh \] && source ~/\.fig/fig\.sh\n?")?
                .replace_all(&contents, "")
                .into();

            if let Some(pre) = self.pre_integration() {
                contents = pre
                    .get_source_regex(false)?
                    .replace_all(&contents, "")
                    .into();
            }

            if let Some(post) = self.post_integration() {
                contents = post
                    .get_source_regex(false)?
                    .replace_all(&contents, "")
                    .into();
            }

            contents = contents.trim().to_string();
            contents.push('\n');

            std::fs::write(&path, contents.as_bytes())?;
        }

        Ok(())
    }

    pub fn install(&self, backup_dir: Option<&Path>) -> Result<()> {
        let path = self.path();
        let mut contents = String::new();
        if path.exists() {
            if let Some(name) = path.file_name() {
                let backup = match backup_dir {
                    Some(backup) => Some(backup.to_path_buf()),
                    None => {
                        if let Ok(now) =
                            OffsetDateTime::now_utc().format(time::macros::format_description!(
                                "[year]-[month]-[day]_[hour]-[minute]-[second]"
                            ))
                        {
                            fig_directories::home_dir()
                                .map(|path| path.join(".fig.dotfiles.bak").join(now))
                        } else {
                            None
                        }
                    }
                };

                if let Some(backup) = backup {
                    std::fs::create_dir_all(backup.as_path()).context("Could not back up file")?;
                    std::fs::copy(path.as_path(), backup.join(name).as_path())
                        .context("Could not back up file")?;
                }
            }

            // Remove existing integration.
            self.uninstall()?;

            if path.exists() {
                let mut file = File::open(&path)?;
                file.read_to_string(&mut contents)?;
            }
        }

        let mut modified = false;
        let mut new_contents = String::new();

        if let Some(integration) = self.pre_integration() {
            if !integration.get_source_regex(false)?.is_match(&contents) {
                new_contents.push_str(&integration.text());
                new_contents.push('\n');
                modified = true;
            }
        }

        new_contents.push_str(&contents);

        if let Some(integration) = self.post_integration() {
            if !integration.get_source_regex(false)?.is_match(&contents) {
                new_contents.push('\n');
                new_contents.push_str(&integration.text());
                new_contents.push('\n');
                modified = true;
            }
        }

        if modified {
            let mut file = File::create(&path)?;
            file.write_all(new_contents.as_bytes())?;
        }

        Ok(())
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

    pub fn get_shell_integrations(&self) -> Result<Vec<ShellFileIntegration>> {
        let home_dir = fig_directories::home_dir().context("Failed to get base directories")?;

        let path = match self {
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
                    .map(|filename| ShellFileIntegration {
                        directory: home_dir.clone(),
                        filename,
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
                    .unwrap_or_else(|_| home_dir);
                vec![".zshrc", ".zprofile"]
                    .into_iter()
                    .map(|filename| ShellFileIntegration {
                        directory: zdotdir.clone(),
                        filename,
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
                        directory: fish_config_dir.clone(),
                        filename: "00_fig_pre.fish",
                        shell: *self,
                        pre: true,
                        post: false,
                        remove_on_uninstall: true,
                    },
                    ShellFileIntegration {
                        directory: fish_config_dir,
                        filename: "99_fig_post.fish",
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

    pub fn get_fig_integration_source(&self, when: &When) -> &'static str {
        match (self, when) {
            (Shell::Fish, When::Pre) => include_str!("../integrations/shell/pre.fish"),
            (Shell::Fish, When::Post) => include_str!("../integrations/shell/post.fish"),
            (Shell::Zsh, When::Pre) => include_str!("../integrations/shell/pre.sh"),
            (Shell::Zsh, When::Post) => include_str!("../integrations/shell/post.zsh"),
            (Shell::Bash, When::Pre) => {
                concat!(
                    "function __fig_source_bash_preexec() {\n",
                    include_str!("../integrations/shell/bash-preexec.sh"),
                    "\n}\n\
                    __fig_source_bash_preexec\n\
                    function __bp_adjust_histcontrol() { :; }\n",
                    include_str!("../integrations/shell/pre.sh")
                )
            }
            (Shell::Bash, When::Post) => {
                concat!(
                    "function __fig_source_bash_preexec() {\n",
                    include_str!("../integrations/shell/bash-preexec.sh"),
                    "\n}\n\
                    __fig_source_bash_preexec\n\
                    function __bp_adjust_histcontrol() { :; }\n",
                    include_str!("../integrations/shell/post.bash")
                )
            }
        }
    }

    pub fn get_data_path(&self) -> Option<PathBuf> {
        fig_directories::fig_data_dir().map(|dir| dir.join("shell").join(format!("{}.json", self)))
    }

    pub fn get_remote_source(&self) -> Result<Url> {
        Ok(format!("{}/dotfiles/source/{}", api_host(), self).parse()?)
    }
}
