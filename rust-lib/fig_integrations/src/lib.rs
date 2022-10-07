pub mod backup;
pub mod error;
pub mod file;
pub mod shell;
pub mod ssh;

use std::path::Path;

pub use backup::backup_file;
pub use error::{
    Error,
    Result,
};
pub use file::FileIntegration;

#[cfg(target_os = "macos")]
pub mod accessibility;

pub trait Integration {
    fn describe(&self) -> String;
    fn install(&self, backup_dir: Option<&Path>) -> Result<()>;
    fn uninstall(&self) -> Result<()>;
    fn is_installed(&self) -> Result<()>;
}
