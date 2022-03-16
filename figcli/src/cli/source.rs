//! Sync of dotfiles

use crate::dotfiles::{
    download_and_notify,
    notify::{notify_terminal, TerminalNotification},
};
use anyhow::Result;

/// Download the lastest dotfiles
pub async fn source_cli() -> Result<()> {
    download_and_notify().await?;
    if let Ok(session_id) = std::env::var("TERM_SESSION_ID") {
        notify_terminal(session_id, TerminalNotification::Source)?;
    }
    Ok(())
}
