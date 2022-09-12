//! Sync of dotfiles

use eyre::{
    Result,
    WrapErr,
};
use fig_auth::is_logged_in;
use fig_sync::dotfiles::download_and_notify;
use fig_sync::dotfiles::notify::{
    notify_terminal,
    TerminalNotification,
};

/// Download the latest dotfiles
pub async fn source_cli() -> Result<()> {
    if !is_logged_in() {
        eyre::bail!("Must be logged in to sync dotfiles");
    }
    download_and_notify(true)
        .await
        .context("Could not sync remote dotfiles")?;
    if let Ok(session_id) = std::env::var("TERM_SESSION_ID") {
        notify_terminal(session_id, TerminalNotification::Source)?;
    }
    Ok(())
}
