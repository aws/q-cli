use fig_util::directories;
use thiserror::Error;

// A cloneable error
#[derive(Debug, Clone, thiserror::Error)]
#[error("Failed to open database: {}", .0)]
pub struct DbOpenError(pub(crate) String);

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error(transparent)]
    FigUtilError(#[from] fig_util::Error),
    #[error("settings file is not a json object")]
    SettingsNotObject,
    #[error(transparent)]
    DirectoryError(#[from] directories::DirectoryError),
    #[error("memory backend is not used")]
    MemoryBackendNotUsed,
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    R2d2(#[from] r2d2::Error),
    #[error(transparent)]
    DbOpenError(#[from] DbOpenError),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
