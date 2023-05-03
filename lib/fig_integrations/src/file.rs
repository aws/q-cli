use std::path::PathBuf;

use async_trait::async_trait;

use crate::error::{
    Error,
    Result,
};
use crate::Integration;

#[derive(Debug, Clone)]
pub struct FileIntegration {
    pub path: PathBuf,
    pub contents: String,
}

#[async_trait]
impl Integration for FileIntegration {
    fn describe(&self) -> String {
        format!("File Integration @ {}", self.path.to_string_lossy())
    }

    async fn is_installed(&self) -> Result<()> {
        let current_contents = std::fs::read_to_string(&self.path)
            .map_err(|_| Error::Custom(format!("{} does not exist.", self.path.display()).into()))?;
        if current_contents.ne(&self.contents) {
            let message = format!("{} should contain:\n{}", self.path.display(), self.contents);
            return Err(Error::ImproperInstallation(message.into()));
        }
        Ok(())
    }

    async fn install(&self) -> Result<()> {
        if self.is_installed().await.is_ok() {
            return Ok(());
        }
        let parent_dir = self
            .path
            .parent()
            .ok_or_else(|| Error::Custom("Could not get integration file directory".into()))?;
        std::fs::create_dir_all(parent_dir)?;

        match std::fs::write(&self.path, &self.contents) {
            Ok(_) => Ok(()),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    Err(Error::PermissionDenied {
                        path: self.path.clone(),
                    })
                } else {
                    Err(e.into())
                }
            },
        }
    }

    async fn uninstall(&self) -> Result<()> {
        if self.path.exists() {
            match std::fs::remove_file(&self.path) {
                Ok(_) => Ok(()),
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::PermissionDenied {
                        Err(Error::PermissionDenied {
                            path: self.path.clone(),
                        })
                    } else {
                        Err(e.into())
                    }
                },
            }
        } else {
            Ok(())
        }
    }
}
