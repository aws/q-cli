use std::time::SystemTimeError;

use cfg_if::cfg_if;
use fig_util::get_system_id;
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

fn system_threshold(version: &str) -> Result<u8, Error> {
    let mut threshold: u8 = 0;

    let mut apply = |from: &str| {
        for ch in from.chars() {
            threshold = threshold.wrapping_add((((ch as u32) % 256) as u8).wrapping_add(128));
        }
    };

    // different for each system
    apply(&get_system_id()?);
    // different for each version
    apply(version);

    Ok(threshold)
}

pub fn apply_update(package: Package) -> Result<(), Error> {
    cfg_if! {
        if #[cfg(windows)] {
            return windows::update(package);
        } else if #[cfg(target_os = "macos")] {
            return macos::update(package),
        } else {
            let _package = package;
            return Err(Error::UnsupportedPlatform)
        }
    }
}
