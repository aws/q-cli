use crate::util::shell::Shell;
use anyhow::{Context, Result};
use clap::ArgEnum;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};
use thiserror::Error;
use time::OffsetDateTime;

#[derive(Debug, Copy, Clone, PartialEq, Eq, ArgEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum When {
    Pre,
    Post,
}

#[derive(Error, Debug)]
pub enum InstallationError {
    #[error("Warning: Legacy integration. {0}")]
    LegacyInstallation(Cow<'static, str>),
    #[error("Error: Improper integration installation. {0}")]
    ImproperInstallation(Cow<'static, str>),
    #[error("Error: Integration not installed. {0}")]
    NotInstalled(Cow<'static, str>),
}

impl From<anyhow::Error> for InstallationError {
    fn from(e: anyhow::Error) -> InstallationError {
        InstallationError::NotInstalled(format!("{}", e).into())
    }
}

impl std::fmt::Display for When {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            When::Pre => write!(f, "pre"),
            When::Post => write!(f, "post"),
        }
    }
}

fn get_default_backup_dir() -> Result<PathBuf> {
    let now = OffsetDateTime::now_utc().format(time::macros::format_description!(
        "[year]-[month]-[day]_[hour]-[minute]-[second]"
    ))?;
    fig_directories::home_dir()
        .map(|path| path.join(".fig.dotfiles.bak").join(now))
        .context("Could not get home dir")
}

fn backup_file<P>(path: P, backup_dir: Option<&Path>) -> Result<()>
where
    P: AsRef<Path>,
{
    let pathref = path.as_ref();
    if pathref.exists() {
        let name: String = pathref
            .file_name()
            .context(format!("Could not get filename for {}", pathref.display()))?
            .to_string_lossy()
            .into_owned();
        let dir = backup_dir
            .map(|dir| dir.to_path_buf())
            .or_else(|| get_default_backup_dir().ok())
            .context("Could not get backup directory")?;
        std::fs::create_dir_all(&dir).context("Could not back up file")?;
        std::fs::copy(path, dir.join(name).as_path()).context("Could not back up file")?;
    }

    Ok(())
}

pub trait ShellIntegration: Send + Sync + ShellIntegrationClone {
    fn install(&self, backup_dir: Option<&Path>) -> Result<()>;
    fn uninstall(&self) -> Result<()>;
    fn is_installed(&self) -> Result<(), InstallationError>;
    fn path(&self) -> PathBuf;
    fn get_shell(&self) -> Shell;
}

impl std::fmt::Display for dyn ShellIntegration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.get_shell(), self.path().display())
    }
}

pub trait ShellIntegrationClone {
    fn clone_box(&self) -> Box<dyn ShellIntegration>;
}

impl<T> ShellIntegrationClone for T
where
    T: 'static + ShellIntegration + Clone,
{
    fn clone_box(&self) -> Box<dyn ShellIntegration> {
        Box::new(self.clone())
    }
}

// We can now implement Clone manually by forwarding to clone_box.
impl Clone for Box<dyn ShellIntegration> {
    fn clone(&self) -> Box<dyn ShellIntegration> {
        self.clone_box()
    }
}

// Fish-like integration where there is a single "integration file"
// that sits in a location picked up by the shell.
#[derive(Debug, Clone)]
pub struct IntegrationFileShellIntegration {
    pub shell: Shell,
    pub when: When,
    pub path: PathBuf,
}

fn get_prefix(s: &str) -> &str {
    match s.find('.') {
        Some(i) => &s[..i],
        None => s,
    }
}

impl IntegrationFileShellIntegration {
    fn get_contents(&self) -> String {
        let rcfile = match self.path.file_name().and_then(|x| x.to_str()) {
            Some(name) => format!(" --rcfile {}", get_prefix(name)),
            None => "".into(),
        };
        match self.shell {
            Shell::Fish => format!(
                "eval (~/.local/bin/fig init {} {}{} | string split0)",
                self.shell, self.when, rcfile
            ),
            _ => format!(
                "eval \"$(~/.local/bin/fig init {} {}{})\"",
                self.shell, self.when, rcfile
            ),
        }
    }
}

impl ShellIntegration for IntegrationFileShellIntegration {
    fn path(&self) -> PathBuf {
        self.path.clone()
    }

    fn get_shell(&self) -> Shell {
        self.shell
    }

    fn is_installed(&self) -> Result<(), InstallationError> {
        let current_contents = std::fs::read_to_string(&self.path)
            .context(format!("{} does not exist.", self.path.display()))?;
        let contents = self.get_contents();
        if current_contents.ne(&contents) {
            let message = format!("{} should contain:\n{}", self.path.display(), contents);
            return Err(InstallationError::ImproperInstallation(message.into()));
        }
        Ok(())
    }

    fn install(&self, _: Option<&Path>) -> Result<()> {
        if self.is_installed().is_ok() {
            return Ok(());
        }
        let parent_dir = self
            .path
            .parent()
            .context("Could not get integration file directory")?;
        std::fs::create_dir_all(parent_dir)?;
        std::fs::write(&self.path, self.get_contents())?;
        Ok(())
    }

    fn uninstall(&self) -> Result<()> {
        if self.path.exists() {
            std::fs::remove_file(&self.path)?;
        }
        Ok(())
    }
}

// zsh and bash integration where we modify a dotfile with pre/post hooks
// that reference integration files.
#[derive(Debug, Clone)]
pub struct DotfileShellIntegration {
    pub shell: Shell,
    pub pre: bool,
    pub post: bool,
    pub dotfile_directory: PathBuf,
    pub dotfile_name: &'static str,
}

impl DotfileShellIntegration {
    fn dotfile_path(&self) -> PathBuf {
        self.dotfile_directory.join(self.dotfile_name)
    }

    fn file_integration(&self, when: When) -> Result<IntegrationFileShellIntegration> {
        let integration_file_name = format!(
            "{}.{}.{}",
            Regex::new(r"^\.")
                .unwrap()
                .replace_all(self.dotfile_name, ""),
            when,
            self.shell
        );
        Ok(IntegrationFileShellIntegration {
            shell: self.shell,
            when,
            path: fig_directories::fig_dir()
                .context("Could not get fig dir")?
                .join("shell")
                .join(integration_file_name),
        })
    }

    fn description(&self, when: When) -> String {
        match when {
            When::Pre => "# Fig pre block. Keep at the top of this file.".into(),
            When::Post => "# Fig post block. Keep at the bottom of this file.".into(),
        }
    }

    fn legacy_regexes(&self, when: When) -> Result<Vec<Regex>> {
        let eval_line = match self.shell {
            Shell::Fish => format!("eval (fig init {} {} | string split0)", self.shell, when),
            _ => format!("eval \"$(fig init {} {})\"", self.shell, when),
        };

        let old_eval_source = match when {
            When::Pre => match self.shell {
                Shell::Fish => format!("set -Ua fish_user_paths $HOME/.local/bin\n{}", eval_line),
                _ => format!(
                    "export PATH=\"${{PATH}}:${{HOME}}/.local/bin\"\n{}",
                    eval_line
                ),
            },
            When::Post => eval_line,
        };

        let old_file_regex = match when {
            When::Pre => {
                Regex::new(r"\[ -s ~/\.fig/shell/pre\.sh \] && source ~/\.fig/shell/pre\.sh\n?")?
            }
            When::Post => Regex::new(r"\[ -s ~/\.fig/fig\.sh \] && source ~/\.fig/fig\.sh\n?")?,
        };
        let old_eval_regex = format!(
            r#"(?:{}\n)?{}\n{{0,2}}"#,
            regex::escape(&self.description(when)),
            regex::escape(&old_eval_source),
        );
        Ok(vec![old_file_regex, Regex::new(&old_eval_regex)?])
    }

    fn source_text(&self, when: When) -> Result<String> {
        let home = fig_directories::home_dir().context("Could not get home dir")?;
        let integration_path = self.file_integration(when)?.path;
        let path = integration_path.strip_prefix(home)?;
        Ok(format!(". \"$HOME/{}\"", path.display()))
    }

    fn source_regex(&self, when: When, constrain_position: bool) -> Result<Regex> {
        let regex = format!(
            r#"{}(?:{}\n)?{}\n{{0,2}}{}"#,
            if constrain_position && when == When::Pre {
                "^"
            } else {
                ""
            },
            regex::escape(&self.description(when)),
            regex::escape(&self.source_text(when)?),
            if constrain_position && when == When::Post {
                "$"
            } else {
                ""
            },
        );
        Regex::new(&regex).context("Invalid source regex")
    }

    fn remove_from_text(&self, text: impl Into<String>, when: When) -> Result<String> {
        let mut regexes = self.legacy_regexes(when)?;
        regexes.push(self.source_regex(when, false)?);
        Ok(regexes
            .iter()
            .fold::<String, _>(text.into(), |acc, reg| reg.replace_all(&acc, "").into()))
    }

    fn matches_text(&self, text: &str, when: When) -> Result<(), InstallationError> {
        let dotfile = self.dotfile_path();
        if self.legacy_regexes(when)?.iter().any(|r| r.is_match(text)) {
            let message = format!("{} has legacy {} integration.", dotfile.display(), when);
            return Err(InstallationError::LegacyInstallation(message.into()));
        }
        if !self.source_regex(when, false)?.is_match(text) {
            let message = format!("{} does not source {} integration", dotfile.display(), when);
            return Err(InstallationError::NotInstalled(message.into()));
        }
        if !self.source_regex(when, true)?.is_match(text) {
            let position = match when {
                When::Pre => "first",
                When::Post => "last",
            };
            let message = format!(
                "{} does not source {} integration {}",
                dotfile.display(),
                when,
                position
            );
            return Err(InstallationError::ImproperInstallation(message.into()));
        }
        Ok(())
    }
}

impl ShellIntegration for DotfileShellIntegration {
    fn get_shell(&self) -> Shell {
        self.shell
    }

    fn path(&self) -> PathBuf {
        self.dotfile_path()
    }

    fn install(&self, backup_dir: Option<&Path>) -> Result<()> {
        if self.is_installed().is_ok() {
            return Ok(());
        }

        let dotfile = self.dotfile_path();
        let mut contents = if dotfile.exists() {
            backup_file(&dotfile, backup_dir)?;
            self.uninstall()?;
            std::fs::read_to_string(&dotfile)?
        } else {
            String::new()
        };

        let original_contents = contents.clone();

        if self.pre {
            self.file_integration(When::Pre)?.install(backup_dir)?;
            contents = format!(
                "{}\n{}\n{}",
                self.description(When::Pre),
                self.source_text(When::Pre)?,
                contents,
            );
        }

        if self.post {
            self.file_integration(When::Post)?.install(backup_dir)?;
            contents = format!(
                "{}\n{}\n{}\n",
                contents,
                self.description(When::Post),
                self.source_text(When::Post)?,
            );
        }

        if contents.ne(&original_contents) {
            let mut file = File::create(&dotfile)?;
            file.write_all(contents.as_bytes())?;
        }

        Ok(())
    }

    fn is_installed(&self) -> Result<(), InstallationError> {
        let dotfile = self.dotfile_path();
        let filtered_contents: String = match std::fs::read_to_string(&dotfile) {
            // Remove comments and empty lines.
            Ok(contents) => Regex::new(r"^\s*(#.*)?\n")
                .unwrap()
                .replace_all(&contents, "")
                .into(),
            _ => {
                let message = format!("{} does not exist.", dotfile.display());
                return Err(InstallationError::NotInstalled(message.into()));
            }
        };

        if self.pre {
            self.matches_text(&filtered_contents, When::Pre)?;
            self.file_integration(When::Pre)?.is_installed()?;
        }

        if self.post {
            self.matches_text(&filtered_contents, When::Post)?;
            self.file_integration(When::Post)?.is_installed()?;
        }

        Ok(())
    }

    fn uninstall(&self) -> Result<()> {
        let dotfile = self.dotfile_path();
        if dotfile.exists() {
            let mut contents = std::fs::read_to_string(&dotfile)?;

            // Remove comment lines
            contents = Regex::new(r"(?mi)^#.*fig.*var.*$\n?")?
                .replace_all(&contents, "")
                .into();

            contents = Regex::new(
                r"(?mi)^#.*Please make sure this block is at the .* of this file.*$\n?",
            )?
            .replace_all(&contents, "")
            .into();

            if self.pre {
                contents = self.remove_from_text(&contents, When::Pre)?;
            }

            if self.post {
                contents = self.remove_from_text(&contents, When::Post)?;
            }

            contents = contents.trim().to_string();
            contents.push('\n');

            std::fs::write(&dotfile, contents.as_bytes())?;
        }

        if self.pre {
            self.file_integration(When::Pre)?.uninstall()?;
        }

        if self.post {
            self.file_integration(When::Post)?.uninstall()?;
        }

        Ok(())
    }
}
