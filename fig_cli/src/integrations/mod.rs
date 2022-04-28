pub mod ssh;
pub mod shell;

use anyhow::{Context, Result};
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};
use thiserror::Error;
use time::OffsetDateTime;

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

fn get_default_backup_dir() -> Result<PathBuf> {
    let now = OffsetDateTime::now_utc().format(time::macros::format_description!(
        "[year]-[month]-[day]_[hour]-[minute]-[second]"
    ))?;
    fig_directories::home_dir()
        .map(|path| path.join(".fig.dotfiles.bak").join(now))
        .context("Could not get home dir")
}

pub fn backup_file<P>(path: P, backup_dir: Option<&Path>) -> Result<()>
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

pub trait Integration {
    fn install(&self, backup_dir: Option<&Path>) -> Result<()>;
    fn uninstall(&self) -> Result<()>;
    fn is_installed(&self) -> Result<(), InstallationError>;
}

#[derive(Debug, Clone)]
pub struct FileIntegration {
    pub path: PathBuf,
    pub contents: String,
}

impl Integration for FileIntegration {
    fn is_installed(&self) -> Result<(), InstallationError> {
        let current_contents = std::fs::read_to_string(&self.path)
            .context(format!("{} does not exist.", self.path.display()))?;
        if current_contents.ne(&self.contents) {
            let message = format!("{} should contain:\n{}", self.path.display(), self.contents);
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
        std::fs::write(&self.path, &self.contents)?;
        Ok(())
    }

    fn uninstall(&self) -> Result<()> {
        if self.path.exists() {
            std::fs::remove_file(&self.path)?;
        }
        Ok(())
    }
}
