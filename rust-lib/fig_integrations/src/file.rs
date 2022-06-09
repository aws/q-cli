use std::path::{
    Path,
    PathBuf,
};

use anyhow::{
    Context,
    Result,
};

use crate::{
    InstallationError,
    Integration,
};

#[derive(Debug, Clone)]
pub struct FileIntegration {
    pub path: PathBuf,
    pub contents: String,
}

impl Integration for FileIntegration {
    fn is_installed(&self) -> Result<(), InstallationError> {
        let current_contents =
            std::fs::read_to_string(&self.path).context(format!("{} does not exist.", self.path.display()))?;
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
        let parent_dir = self.path.parent().context("Could not get integration file directory")?;
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
