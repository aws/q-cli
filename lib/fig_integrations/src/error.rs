use std::borrow::Cow;
use std::path::Path;

use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("Legacy integration: {0}")]
    LegacyInstallation(Cow<'static, str>),
    #[error("Improper integration installation: {0}")]
    ImproperInstallation(Cow<'static, str>),
    #[error("Integration not installed: {0}")]
    NotInstalled(Cow<'static, str>),
    #[error("File does not exist: {}", .0.to_string_lossy())]
    FileDoesNotExist(Cow<'static, Path>),
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Dir(#[from] fig_util::directories::DirectoryError),
    #[error("Regex Error: {0}")]
    Regex(#[from] regex::Error),
    #[error(transparent)]
    StripPrefix(#[from] std::path::StripPrefixError),
    #[error("{0}")]
    Custom(Cow<'static, str>),
    #[cfg(target_os = "macos")]
    #[error(transparent)]
    InputMethod(#[from] crate::input_method::InputMethodError),
    #[cfg(target_os = "macos")]
    #[error("Application not installed: {0}")]
    ApplicationNotInstalled(Cow<'static, str>),
    #[error(transparent)]
    SerdeJSON(#[from] serde_json::Error),
    #[cfg(target_os = "macos")]
    #[error(transparent)]
    PList(#[from] plist::Error),
}
