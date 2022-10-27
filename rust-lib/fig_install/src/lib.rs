#[cfg(target_os = "freebsd")]
mod freebsd;
pub mod index;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
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
#[cfg(target_os = "macos")]
pub use os::uninstall_terminal_integrations;
use thiserror::Error;
use tokio::sync::mpsc::Receiver;
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
    #[error("failed to update fig: `{0}`")]
    UpdateFailedPermissions(String),
    // #[cfg(target_os = "macos")]
    #[error("failed to update fig due to auth error: `{0}`")]
    SecurityFramework(#[from] security_framework::base::Error),
    #[error("your system is not supported on this channel")]
    SystemNotOnChannel,
    #[error("manifest not found")]
    ManifestNotFound,
    #[error("update in progress")]
    UpdateInProgress,
    #[error("could not convert path to cstring")]
    Nul(#[from] std::ffi::NulError),
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

pub async fn check_for_updates(disable_rollout: bool) -> Result<Option<UpdatePackage>, Error> {
    let manifest = manifest().as_ref().ok_or(Error::ManifestNotFound)?;

    index::check_for_updates(
        get_channel()?,
        manifest.kind.clone(),
        manifest.variant.clone(),
        disable_rollout,
    )
    .await
}

#[derive(Debug, Clone)]
pub enum UpdateStatus {
    Percent(f32),
    Message(String),
    Error(String),
    Exit,
}

/// Attempt to update if there is a newer version of Fig
pub async fn update(
    deprecated_no_confirm: bool,
    on_update: Option<Box<dyn FnOnce(Receiver<UpdateStatus>) + Send>>,
    disable_rollout: bool,
) -> Result<bool, Error> {
    info!("Checking for updates...");
    if let Some(update) = check_for_updates(disable_rollout).await? {
        info!("Found update: {}", update.version);

        let (tx, rx) = tokio::sync::mpsc::channel(16);

        let lock_file = fig_util::directories::update_lock_path()?;

        let now_unix_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // If the lock file is older than 1hr, we can assume it's stale and remove it
        if lock_file.exists() {
            match std::fs::read_to_string(&lock_file) {
                Ok(contents) => {
                    let lock_unix_time = contents.parse::<u64>().unwrap_or(0);
                    if now_unix_time - lock_unix_time < 3600 {
                        return Err(Error::UpdateInProgress);
                    } else {
                        std::fs::remove_file(&lock_file)?;
                    }
                },
                Err(err) => {
                    error!(%err, "Failed to read lock file, but it exists");
                },
            }
        }

        tokio::fs::write(&lock_file, &format!("{now_unix_time}")).await?;

        let join = tokio::spawn(async move {
            tx.send(UpdateStatus::Message("Starting Update...".into())).await.ok();
            if let Err(err) = os::update(update, deprecated_no_confirm, tx.clone()).await {
                error!(%err, "Failed to update");

                if let Err(err) = tokio::fs::remove_file(&lock_file).await {
                    error!(%err, "Failed to remove lock file");
                }

                let err_id = fig_telemetry::sentry::capture_error(&err);

                tx.send(UpdateStatus::Error(format!("{err}\nError ID: {err_id}")))
                    .await
                    .ok();

                return Err(err);
            }
            tokio::fs::remove_file(&lock_file).await?;
            Ok(())
        });

        if let Some(on_update) = on_update {
            info!("Updating Fig...");
            on_update(rx);
        } else {
            drop(rx);
        }

        join.await.expect("Failed to join update thread")?;
        Ok(true)
    } else {
        info!("No updates available");
        Ok(false)
    }
}

pub fn get_uninstall_url() -> String {
    // Open the uninstallation page
    let os = std::env::consts::OS;
    let email = fig_request::auth::get_email().unwrap_or_default();
    let version = env!("CARGO_PKG_VERSION");
    format!("https://fig.io/uninstall?email={email}&version={version}&os={os}")
}
