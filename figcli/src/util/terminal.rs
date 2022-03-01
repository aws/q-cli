use clap::ArgEnum;
use serde::{Deserialize, Serialize};

/// Terminals supported by Fig
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ArgEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Terminal {
    /// iTerm 2
    Iterm,
    /// Native macOS terminal
    TerminalApp,
    /// Hyper terminal
    Hyper,
    /// Alacritty terminal
    Alacritty,
    /// Kitty terminal
    Kitty,
    /// VSCode terminal
    Vscode,
}

impl Terminal {
    /// Get the bundle identifier for the terminal
    pub fn to_bundle_id(&self) -> String {
        match self {
            Terminal::Iterm => String::from("com.googlecode.iterm2"),
            Terminal::TerminalApp => String::from("com.googlecode.iterm2"),
            Terminal::Hyper => String::from("com.zeit.hyper"),
            Terminal::Alacritty => String::from("com.alacritty"),
            Terminal::Kitty => String::from("net.kovidgoyal.kitty"),
            Terminal::Vscode => String::from("com.microsoft.VSCode"),
        }
    }
}
