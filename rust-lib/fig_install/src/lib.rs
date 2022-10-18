#[cfg(target_os = "freebsd")]
mod freebsd;
pub mod index;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

use std::str::FromStr;
use std::time::SystemTimeError;

use fig_util::manifest::{
    manifest,
    Channel,
};
#[cfg(target_os = "freebsd")]
use freebsd as os;
use index::UpdatePackage;
#[cfg(target_os = "linux")]
use linux as os;
#[cfg(target_os = "macos")]
use macos as os;
use thiserror::Error;
use tracing::{
    error,
    info,
};
#[cfg(windows)]
use windows as os;

mod common;
pub use common::{
    install,
    uninstall,
    InstallComponents,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("unsupported platform")]
    UnsupportedPlatform,
    #[error(transparent)]
    Util(#[from] fig_util::Error),
    #[error(transparent)]
    Integration(#[from] fig_integrations::Error),
    #[error(transparent)]
    Daemon(#[from] fig_daemon::Error),
    #[error(transparent)]
    Settings(#[from] fig_settings::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("error converting path")]
    PathConversionError(#[from] camino::FromPathBufError),
    #[error(transparent)]
    Semver(#[from] semver::Error),
    #[error(transparent)]
    SystemTime(#[from] SystemTimeError),
    #[error(transparent)]
    Strum(#[from] strum::ParseError),
    #[error("could not determine fig version")]
    UnclearVersion,
    #[error("please update from your package manager")]
    PackageManaged,
    #[error("failed to update fig: `{0}`")]
    UpdateFailed(String),
    #[error("your system is not supported on this channel")]
    SystemNotOnChannel,
    #[error("manifest not found")]
    ManifestNotFound,
}

impl From<fig_util::directories::DirectoryError> for Error {
    fn from(err: fig_util::directories::DirectoryError) -> Self {
        fig_util::Error::Directory(err).into()
    }
}

pub fn get_channel() -> Result<Channel, Error> {
    Ok(match fig_settings::state::get_string("updates.channel")? {
        Some(channel) => Channel::from_str(&channel)?,
        None => manifest()
            .as_ref()
            .ok_or(Error::ManifestNotFound)?
            .default_channel
            .clone(),
    })
}

pub async fn check_for_updates() -> Result<Option<UpdatePackage>, Error> {
    let manifest = manifest().as_ref().ok_or(Error::ManifestNotFound)?;

    index::check_for_updates(get_channel()?, manifest.kind.clone(), manifest.variant.clone()).await
}

/// Attempt to update if there is a newer version of Fig
pub async fn update(deprecated_no_confirm: bool) -> Result<(), Error> {
    info!("Checking for updates...");
    if let Some(update) = check_for_updates().await? {
        info!("Found update: {}", update.version);
        os::update(update, deprecated_no_confirm).await?;
    } else {
        info!("No updates available");
    }

    Ok(())
}

pub fn get_uninstall_url() -> String {
    // Open the uninstallation page
    let os = std::env::consts::OS;
    let email = fig_request::auth::get_email().unwrap_or_default();
    let version = env!("CARGO_PKG_VERSION");
    format!("https://fig.io/uninstall?email={email}&version={version}&os={os}")
}
