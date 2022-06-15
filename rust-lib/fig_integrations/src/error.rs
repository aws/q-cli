use std::borrow::Cow;
use std::path::Path;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum InstallationError {
    #[error("Legacy integration: {0}")]
    LegacyInstallation(Cow<'static, str>),
    #[error("Improper integration installation: {0}")]
    ImproperInstallation(Cow<'static, str>),
    #[error("Integration not installed: {0}")]
    NotInstalled(Cow<'static, str>),
    #[error("File does not exist: {0:?}")]
    FileDoesNotExist(Cow<'static, Path>),
}

impl From<anyhow::Error> for InstallationError {
    fn from(e: anyhow::Error) -> InstallationError {
        InstallationError::NotInstalled(e.to_string().into())
    }
}
