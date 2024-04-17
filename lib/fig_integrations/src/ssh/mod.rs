use std::fs::{
    DirBuilder,
    File,
};
use std::io::Write;
use std::os::unix::fs::DirBuilderExt;
use std::path::PathBuf;

use async_trait::async_trait;
use fig_util::consts::CLI_BINARY_NAME;
use fig_util::directories::{
    self,
    fig_data_dir_utf8,
    home_dir,
};
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

#[derive(Debug, Clone)]
pub struct SshIntegration {
    path: PathBuf,
}

impl SshIntegration {
    pub fn new() -> Result<Self, Error> {
        let path = home_dir()?.join(".ssh").join("config");
        Ok(SshIntegration { path })
    }

    #[allow(clippy::unused_self)]
    fn get_integration_path(&self) -> Result<PathBuf> {
        Ok(directories::fig_data_dir()?.join("ssh"))
    }

    fn get_file_integration(&self) -> Result<FileIntegration> {
        let bin_name = CLI_BINARY_NAME;
        let include_path = fig_data_dir_utf8()?.join("ssh_inner");

        Ok(FileIntegration {
            path: self.get_integration_path()?,
            contents: indoc::formatdoc! {"
                Match exec=\"command -v {bin_name} && {bin_name} internal generate-ssh --remote-host %h --remote-port %p --remote-username %r\"
                    Include \"{include_path}\"
            "},
            #[cfg(unix)]
            mode: Some(0o600),
        })
    }

    #[allow(clippy::unused_self)]
    fn description(&self) -> String {
        "# CodeWhisperer SSH Integration. Keep at the bottom of this file.".into()
    }

    fn source_text(&self) -> Result<String> {
        let home = home_dir()?;
        let integration_path = self.get_integration_path()?;
        let path = integration_path.strip_prefix(home)?;
        Ok(format!("Match all\n  Include \"~/{}\"", path.display()))
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
        self.get_file_integration()?.install().await?;

        if self.is_installed().await.is_ok() {
            return Ok(());
        }

        // Create the .ssh directory if it doesn't exist
        if let Some(path) = self.path.parent() {
            if !path.exists() {
                let mut builder = DirBuilder::new();
                builder.recursive(true);
                #[cfg(unix)]
                builder.mode(0o700);
                builder.create(path)?;
            }
        }

        let contents = if self.path.exists() {
            backup_file(&self.path, fig_util::directories::utc_backup_dir().ok())?;
            self.uninstall().await?;
            std::fs::read_to_string(&self.path)?
        } else {
            String::new()
        };

        let new_contents = format!("{}\n{}\n{}\n", contents, self.description(), self.source_text()?);
        let mut file = File::create(&self.path)?;
        file.write_all(new_contents.as_bytes())?;

        Ok(())
    }

    async fn uninstall(&self) -> Result<()> {
        if self.path.exists() {
            let mut contents = std::fs::read_to_string(&self.path)?;
            contents = self.source_regex()?.replace_all(&contents, "").into();
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
            let message = format!(
                "{} does not source CodeWhisperer's ssh integration",
                self.path.display()
            );
            return Err(Error::NotInstalled(message.into()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_integration() {
        let integration = SshIntegration::new().unwrap();
        let file_integration = integration.get_file_integration().unwrap();

        println!("=== source_text ===");
        println!("{}", integration.source_text().unwrap());
        println!("===================");
        println!();
        println!("=== file_integration ===");
        println!("{}", file_integration.contents);
        println!("========================");
    }
}
