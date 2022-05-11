//! Sync of dotfiles

use anyhow::{
    Context,
    Result,
};
use fig_auth::is_logged_in;

use crate::dotfiles::download_and_notify;
use crate::dotfiles::notify::{
    notify_terminal,
    TerminalNotification,
};

/// Download the lastest dotfiles
pub async fn source_cli() -> Result<()> {
    if !is_logged_in() {
        anyhow::bail!("Must be logged in to sync dotfiles");
    }
    download_and_notify().await.context("Could not sync remote dotfiles")?;
    if let Ok(session_id) = std::env::var("TERM_SESSION_ID") {
        notify_terminal(session_id, TerminalNotification::Source)?;
    }
    Ok(())
}
