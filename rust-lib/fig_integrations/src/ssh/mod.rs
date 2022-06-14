use std::fs::File;
use std::io::Write;
use std::path::{
    Path,
    PathBuf,
};

use anyhow::{
    Context,
    Result,
};
use regex::Regex;

use crate::{
    backup_file,
    FileIntegration,
    InstallationError,
    Integration,
};

#[derive(Debug, Clone)]
pub struct SshIntegration {
    pub path: PathBuf,
}

impl SshIntegration {
    fn get_integration_path(&self) -> Result<PathBuf> {
        Ok(fig_directories::fig_dir()
            .context("Could not get fig dir path")?
            .join("ssh"))
    }

    fn get_file_integration(&self) -> Result<FileIntegration> {
        Ok(FileIntegration {
            path: self.get_integration_path()?,
            contents: include_str!("./ssh_config").into(),
        })
    }

    fn source_text(&self) -> Result<String> {
        let home = fig_directories::home_dir().context("Could not get home dir")?;
        let integration_path = self.get_integration_path()?;
        let path = integration_path.strip_prefix(home)?;
        Ok(format!("Include ~/{}", path.display()))
    }

    fn source_regex(&self) -> Result<Regex> {
        let regex = format!(r#"{}\n{{0,2}}"#, regex::escape(&self.source_text()?));
        Regex::new(&regex).context("Invalid source regex")
    }
}

impl Integration for SshIntegration {
    fn install(&self, backup_dir: Option<&Path>) -> Result<()> {
        if self.is_installed().is_ok() {
            return Ok(());
        }

        let contents = if self.path.exists() {
            backup_file(&self.path, backup_dir)?;
            self.uninstall()?;
            std::fs::read_to_string(&self.path)?
        } else {
            String::new()
        };

        self.get_file_integration()?.install(backup_dir)?;
        let new_contents = format!("{}\n{}\n", contents, self.source_text()?);
        let mut file = File::create(&self.path)?;
        file.write_all(new_contents.as_bytes())?;

        Ok(())
    }

    fn is_installed(&self) -> Result<(), InstallationError> {
        let filtered_contents: String = match std::fs::read_to_string(&self.path) {
            // Remove comments and empty lines.
            Ok(contents) => Regex::new(r"^\s*(#.*)?\n").unwrap().replace_all(&contents, "").into(),
            _ => {
                let message = format!("{} does not exist.", self.path.display());
                return Err(InstallationError::NotInstalled(message.into()));
            },
        };

        self.get_file_integration()?.is_installed()?;
        if !self.source_regex()?.is_match(&filtered_contents) {
            let message = format!("{} does not source Fig's ssh integration", self.path.display());
            return Err(InstallationError::NotInstalled(message.into()));
        }

        Ok(())
    }

    fn uninstall(&self) -> Result<()> {
        if self.path.exists() {
            let mut contents = std::fs::read_to_string(&self.path)?;
            contents = self.source_regex()?.replace_all(&contents, "").into();
            contents = contents.trim().to_string();
            contents.push('\n');
            std::fs::write(&self.path, contents.as_bytes())?;
        }

        self.get_file_integration()?.uninstall()?;

        Ok(())
    }
}
