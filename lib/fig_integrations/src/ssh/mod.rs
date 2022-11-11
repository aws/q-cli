use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use async_trait::async_trait;
use fig_util::directories;
use regex::Regex;

use crate::error::{
    Error,
    Result,
};
use crate::{
    backup_file,
    FileIntegration,
    Integration,
};

pub static SSH_CONFIG_INNER: &str = include_str!("./ssh_config_inner");
pub static SSH_CONFIG_EMPTY: &str = include_str!("./ssh_config_empty");

#[derive(Debug, Clone)]
pub struct SshIntegration {
    path: PathBuf,
}

impl SshIntegration {
    pub fn new() -> Result<Self, Error> {
        let path = directories::home_dir()?.join(".ssh").join("config");
        Ok(SshIntegration { path })
    }

    fn get_integration_path(&self) -> Result<PathBuf> {
        Ok(directories::fig_dir()?.join("ssh"))
    }

    fn get_file_integration(&self) -> Result<FileIntegration> {
        Ok(FileIntegration {
            path: self.get_integration_path()?,
            contents: include_str!("./ssh_config").into(),
        })
    }

    fn legacy_text(&self) -> Result<String> {
        let home = directories::home_dir()?;
        let integration_path = self.get_integration_path()?;
        let path = integration_path.strip_prefix(home)?;
        Ok(format!("Include ~/{}", path.display()))
    }

    fn legacy_regex(&self) -> Result<Regex> {
        let regex = format!(r#"{}\n{{0,2}}"#, regex::escape(&self.legacy_text()?));
        Ok(Regex::new(&regex)?)
    }

    fn description(&self) -> String {
        "# Fig ssh integration. Keep at the bottom of this file.".into()
    }

    fn source_text(&self) -> Result<String> {
        let home = directories::home_dir()?;
        let integration_path = self.get_integration_path()?;
        let path = integration_path.strip_prefix(home)?;
        Ok(format!("Match all\n  Include ~/{}", path.display()))
    }

    fn source_regex(&self) -> Result<Regex> {
        let regex = format!(
            r#"(?:{}\n)?{}\n{{0,2}}"#,
            regex::escape(&self.description()),
            regex::escape(&self.source_text()?)
        );
        Ok(Regex::new(&regex)?)
    }

    pub async fn reinstall(&self) -> Result<()> {
        if self.get_integration_path()?.exists() {
            self.get_file_integration()?.install().await?;
        }
        Ok(())
    }
}

#[async_trait]
impl Integration for SshIntegration {
    fn describe(&self) -> String {
        "SSH Integration".to_owned()
    }

    async fn install(&self) -> Result<()> {
        if self.is_installed().await.is_ok() {
            return Ok(());
        }

        let contents = if self.path.exists() {
            backup_file(&self.path, fig_util::directories::utc_backup_dir().ok())?;
            self.uninstall().await?;
            std::fs::read_to_string(&self.path)?
        } else {
            String::new()
        };

        self.get_file_integration()?.install().await?;
        let new_contents = format!("{}\n{}\n{}\n", contents, self.description(), self.source_text()?);
        let mut file = File::create(&self.path)?;
        file.write_all(new_contents.as_bytes())?;

        Ok(())
    }

    async fn uninstall(&self) -> Result<()> {
        if self.path.exists() {
            let mut contents = std::fs::read_to_string(&self.path)?;
            contents = self.source_regex()?.replace_all(&contents, "").into();
            contents = self.legacy_regex()?.replace_all(&contents, "").into();
            contents = contents.trim().to_string();
            contents.push('\n');
            std::fs::write(&self.path, contents.as_bytes())?;
        }

        self.get_file_integration()?.uninstall().await?;

        Ok(())
    }

    async fn is_installed(&self) -> Result<()> {
        let filtered_contents: String = match std::fs::read_to_string(&self.path) {
            // Remove comments and empty lines.
            Ok(contents) => Regex::new(r"^\s*(#.*)?\n").unwrap().replace_all(&contents, "").into(),
            _ => {
                let message = format!("{} does not exist.", self.path.display());
                return Err(Error::NotInstalled(message.into()));
            },
        };

        self.get_file_integration()?.is_installed().await?;
        if !self.source_regex()?.is_match(&filtered_contents) {
            let message = format!("{} does not source Fig's ssh integration", self.path.display());
            return Err(Error::NotInstalled(message.into()));
        }

        Ok(())
    }
}
