#[cfg(target_os = "freebsd")]
mod freebsd;
pub mod index;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

use std::time::SystemTimeError;

#[cfg(target_os = "freebsd")]
use freebsd as os;
#[cfg(target_os = "linux")]
use linux as os;
#[cfg(target_os = "macos")]
use macos as os;
use thiserror::Error;
#[cfg(windows)]
use windows as os;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
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
    #[error("could not determine fig version")]
    UnclearVersion,
    #[error("please update from your package manager")]
    PackageManaged,
    #[error("failed to update fig: `{0}`")]
    LegacyUpdateFailed(String),
}

pub async fn check_for_updates() -> Result<Option<String>, Error> {
    Ok(index::check_for_updates(env!("CARGO_PKG_VERSION"))
        .await?
        .map(|update| update.version))
}

/// Attempt to update if there is a newer version of Fig
pub async fn update(deprecated_no_confirm: bool) -> Result<(), Error> {
    if let Some(update) = index::check_for_updates(env!("CARGO_PKG_VERSION")).await? {
        os::update(update, deprecated_no_confirm).await?;
    }

    Ok(())
}
