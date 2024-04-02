use std::io::ErrorKind;
use std::path::PathBuf;

use async_trait::async_trait;
use tokio::io::AsyncWriteExt;

use crate::error::{
    Error,
    Result,
};
use crate::Integration;

#[derive(Debug, Clone)]
pub struct FileIntegration {
    pub path: PathBuf,
    pub contents: String,
    #[cfg(unix)]
    pub mode: Option<u32>,
}

#[async_trait]
impl Integration for FileIntegration {
    fn describe(&self) -> String {
        format!("File Integration @ {}", self.path.to_string_lossy())
    }

    async fn is_installed(&self) -> Result<()> {
        let current_contents = tokio::fs::read_to_string(&self.path)
            .await
            .map_err(|err| Error::Custom(format!("{} does not exist: {err}", self.path.display()).into()))?;
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
        if !parent_dir.is_dir() {
            tokio::fs::create_dir_all(parent_dir).await?;
        }

        let mut options = tokio::fs::File::options();
        options.write(true).create(true).truncate(true);

        #[cfg(unix)]
        if let Some(mode) = self.mode {
            options.mode(mode);
        }

        match options.open(&self.path).await {
            Ok(mut file) => {
                file.write_all(self.contents.as_bytes()).await?;
                Ok(())
            },
            Err(err) if err.kind() == ErrorKind::PermissionDenied => Err(Error::PermissionDenied {
                path: self.path.clone(),
            }),
            Err(err) => Err(err.into()),
        }
    }

    async fn uninstall(&self) -> Result<()> {
        if self.path.exists() {
            match tokio::fs::remove_file(&self.path).await {
                Ok(_) => Ok(()),
                Err(err) if err.kind() == ErrorKind::PermissionDenied => Err(Error::PermissionDenied {
                    path: self.path.clone(),
                }),
                Err(err) => Err(err.into()),
            }
        } else {
            Ok(())
        }
    }
}
