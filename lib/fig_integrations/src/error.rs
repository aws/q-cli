use std::borrow::Cow;
use std::path::{
    Path,
    PathBuf,
};

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
    #[error("Permission denied: {}", .path.display())]
    PermissionDenied { path: PathBuf },

    #[error("{context}: {error}")]
    Context {
        #[source]
        error: Box<Self>,
        context: Cow<'static, str>,
    },
}

impl Error {
    pub fn verbose_message(&self) -> String {
        match self {
            Self::PermissionDenied { path } => {
                format!(
                    "Permission denied to write to {path}\nTry running: sudo chown $USER '{path}' && sudo chmod 644 '{path}'",
                    path = path.display()
                )
            },
            err => err.to_string(),
        }
    }
}

pub(crate) trait ErrorExt<T, E> {
    fn context(self, context: impl Into<Cow<'static, str>>) -> Result<T, Error>;
    fn with_context(self, context_fn: impl FnOnce(&E) -> String) -> Result<T, Error>;
}

impl<T, E: Into<Error>> ErrorExt<T, E> for Result<T, E> {
    fn context(self, context: impl Into<Cow<'static, str>>) -> Result<T, Error> {
        self.map_err(|err| {
            let context = context.into();
            let error = err.into();
            Error::Context {
                error: Box::new(error),
                context,
            }
        })
    }

    fn with_context(self, context_fn: impl FnOnce(&E) -> String) -> Result<T, Error> {
        self.map_err(|err| {
            let context = context_fn(&err);
            let error = err.into();
            Error::Context {
                error: Box::new(error),
                context: context.into(),
            }
        })
    }
}
