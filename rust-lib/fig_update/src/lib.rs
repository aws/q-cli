use std::time::SystemTimeError;

use cfg_if::cfg_if;
use thiserror::Error;

pub mod index;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

pub use index::check as check_for_updates;
use index::Package;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unsupported platform")]
    UnsupportedPlatform,
    #[error(transparent)]
    Util(#[from] fig_util::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Semver(#[from] semver::Error),
    #[error(transparent)]
    SystemTime(#[from] SystemTimeError),
}

#[allow(clippy::needless_return)] // actually fairly needed
pub fn apply_update(package: Package) -> Result<(), Error> {
    cfg_if! {
        if #[cfg(target_os = "windows")] {
            return windows::update(package);
        } else if #[cfg(target_os = "macos")] {
            return macos::update(package);
        } else {
            let _package = package;
            return Err(Error::UnsupportedPlatform);
        }
    }
}
