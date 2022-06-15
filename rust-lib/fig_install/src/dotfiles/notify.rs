use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalNotification {
    NewUpdates,
    Source,
}

impl std::str::FromStr for TerminalNotification {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "newUpdates" => Ok(TerminalNotification::NewUpdates),
            "source" => Ok(TerminalNotification::Source),
            _ => Err(anyhow::anyhow!("Invalid terminal notification: {}", s)),
        }
    }
}

impl std::fmt::Display for TerminalNotification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TerminalNotification::NewUpdates => write!(f, "newUpdates"),
            TerminalNotification::Source => write!(f, "source"),
        }
    }
}

pub fn notify_terminal(session_id: impl AsRef<str>, notification: TerminalNotification) -> Result<()> {
    let dotfiles_update_path = std::env::temp_dir()
        .join("fig")
        .join("dotfiles_updates")
        .join(session_id.as_ref());

    std::fs::write(dotfiles_update_path, notification.to_string())?;

    Ok(())
}

/// Notify dotfiles updates
pub fn notify_all_terminals(notification: TerminalNotification) -> Result<()> {
    let tempdir = std::env::temp_dir();
    let dotfiles_updates_folder = tempdir.join("fig").join("dotfiles_updates");

    // Write true to all files in the dotfiles_updates folder
    if dotfiles_updates_folder.exists() {
        for file in dotfiles_updates_folder.read_dir()? {
            let file = file?;

            std::fs::write(file.path(), notification.to_string())?;
        }
    }

    Ok(())
}
