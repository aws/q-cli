pub mod api;
pub mod notify;

use anyhow::Result;
use tracing::{
    error,
    info,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncWhen {
    /// Sync the dotfiles immediately
    Immediately,
    /// Save to a temporary file and sync later
    Later,
}

/// Download and notify terminals about new dotfiles updates bases on the
/// user's settings
pub async fn download_and_notify(always_download: bool) -> Result<Option<api::UpdateStatus>> {
    // Guard if the user has disabled immediate syncing
    if always_download || !fig_settings::settings::get_bool_or("dotfiles.syncImmediately", true) {
        return Ok(None);
    }

    let res = api::download_dotfiles().await;
    match &res {
        Ok(api::UpdateStatus::New) => {
            info!("Dotfiles downloaded for the first time");
        },
        Ok(api::UpdateStatus::Updated) => {
            info!("Dotfiles updated");
            notify::notify_all_terminals(notify::TerminalNotification::NewUpdates)?;
        },
        Ok(api::UpdateStatus::NotUpdated) => {
            info!("Dotfiles are up to date");
        },
        Err(err) => error!("Could not sync dotfiles: {err:?}"),
    }
    res.map(Some)
}
