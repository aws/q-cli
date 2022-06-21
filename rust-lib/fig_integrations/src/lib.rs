pub mod backup;
pub mod error;
pub mod file;
pub mod ibus;
pub mod shell;
pub mod ssh;

use std::path::Path;

use anyhow::Result;
pub use backup::{
    backup_file,
    get_default_backup_dir,
};
pub use error::InstallationError;
pub use file::FileIntegration;

pub trait Integration {
    fn install(&self, backup_dir: Option<&Path>) -> Result<()>;
    fn uninstall(&self) -> Result<()>;
    fn is_installed(&self) -> Result<(), InstallationError>;
}
