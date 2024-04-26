use std::io::ErrorKind;
use std::path::PathBuf;

use async_trait::async_trait;
use tokio::fs::{
    self,
    File,
};
use tokio::io::AsyncWriteExt;
use tracing::debug;

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
        let current_contents = match fs::read_to_string(&self.path).await {
            Ok(contents) => contents,
            Err(err) if err.kind() == ErrorKind::NotFound => {
                return Err(Error::FileDoesNotExist(self.path.clone().into()));
            },
            Err(err) => return Err(err.into()),
        };
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
            fs::create_dir_all(parent_dir).await?;
        }

        let mut options = File::options();
        options.write(true).create(true).truncate(true);

        #[cfg(unix)]
        if let Some(mode) = self.mode {
            options.mode(mode);
        }

        match options.open(&self.path).await {
            Ok(mut file) => {
                debug!(path =? self.path, "Writing file integrations");
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
        match fs::remove_file(&self.path).await {
            Ok(_) => Ok(()),
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
            Err(err) if err.kind() == ErrorKind::PermissionDenied => Err(Error::PermissionDenied {
                path: self.path.clone(),
            }),
            Err(err) => Err(err.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_integration() {
        let tempdir = tempfile::tempdir().unwrap();
        let integration = FileIntegration {
            path: tempdir.path().join("integration.txt"),
            contents: "test".into(),
            // weird mode for testing
            #[cfg(unix)]
            mode: None,
        };

        assert_eq!(
            format!("File Integration @ {}/integration.txt", tempdir.path().display()),
            integration.describe()
        );

        // ensure no intgration is marked as not installed
        assert!(matches!(
            integration.is_installed().await,
            Err(Error::FileDoesNotExist(_))
        ));

        // ensure the intgration can be installed
        integration.install().await.unwrap();
        assert!(integration.is_installed().await.is_ok());

        // ensure the intgration can be installed while already installed
        integration.install().await.unwrap();
        assert!(integration.is_installed().await.is_ok());

        // ensure the intgration can be uninstalled
        integration.uninstall().await.unwrap();
        assert!(matches!(
            integration.is_installed().await,
            Err(Error::FileDoesNotExist(_))
        ));

        // write bad data to integration file
        fs::write(&integration.path, "bad data").await.unwrap();
        assert!(matches!(
            integration.is_installed().await,
            Err(Error::ImproperInstallation(_))
        ));

        // fix integration file
        integration.install().await.unwrap();
        assert!(integration.is_installed().await.is_ok());
    }
}
