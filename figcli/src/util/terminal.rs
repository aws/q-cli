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
    /// VSCode Insiders
    VSCodeInsiders,
    /// Tabby
    Tabby,
}

impl Terminal {
    /// Get the bundle identifier for the terminal
    pub fn to_bundle_id(&self) -> String {
        match self {
            Terminal::Iterm => String::from("com.googlecode.iterm2"),
            Terminal::TerminalApp => String::from("com.apple.Terminal"),
            Terminal::Hyper => String::from("com.zeit.hyper"),
            Terminal::Alacritty => String::from("com.alacritty"),
            Terminal::Kitty => String::from("net.kovidgoyal.kitty"),
            Terminal::Vscode => String::from("com.microsoft.VSCode"),
            Terminal::VSCodeInsiders => String::from("com.microsoft.VSCodeInsiders"),
            Terminal::Tabby => String::from("org.tabby"),
        }
    }

    pub fn current_terminal() -> Option<Self> {
        match std::env::var("TERM_PROGRAM").ok().as_deref() {
            Some("iTerm.app") => Some(Terminal::Iterm),
            Some("Apple_Terminal") => Some(Terminal::TerminalApp),
            Some("Hyper") => Some(Terminal::Hyper),
            Some("vscode") => match std::env::var("TERM_PROGRAM_VERSION").ok().as_deref() {
                Some(v) if v.contains("insiders") => Some(Terminal::VSCodeInsiders),
                _ => Some(Terminal::Vscode),
            },
            Some("Tabby") => Some(Terminal::Tabby),
            _ => match std::env::var("__CFBundleIdentifier").ok().as_deref() {
                // Add support for Jetbrain Terminals
                // Some(v) if v.contains("com.jetbrains.") => Some(Terminal::JediTerm),
                _ => None,
            },
        }
    }

    pub fn is_jetbrains_terminal() -> bool {
        // Handles all official JetBrain IDEs + Android Studio
        matches!(std::env::var("TERMINAL_EMULATOR").ok(), Some(v) if v == "JetBrains-JediTerm")
    }
}
