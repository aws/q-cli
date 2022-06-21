use std::borrow::Cow;
use std::fmt;

use serde::{
    Deserialize,
    Serialize,
};

pub const MACOS_TERMINALS: &[Terminal] = &[];
pub const LINUX_TERMINALS: &[Terminal] = &[
    Terminal::Vscode,
    Terminal::VSCodeInsiders,
    Terminal::GnomeTerminal,
    Terminal::Konsole,
    Terminal::Tilix,
    Terminal::Alacritty,
    Terminal::Kitty,
    Terminal::XfceTerminal,
    Terminal::Terminator,
];

/// Terminals supported by Fig
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    /// Nova
    Nova,
    /// Wezterm
    WezTerm,
    /// Jetbrains Terminal
    JediTerm(String),
    /// Gnome Terminal
    GnomeTerminal,
    /// KDE Konsole
    Konsole,
    /// Tilix
    Tilix,
    /// Xfce Terminal
    XfceTerminal,
    /// Terminator
    Terminator,
}

impl fmt::Display for Terminal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Terminal::Iterm => write!(f, "iTerm 2"),
            Terminal::TerminalApp => write!(f, "macOS"),
            Terminal::Hyper => write!(f, "Hyper"),
            Terminal::Alacritty => write!(f, "Alacritty"),
            Terminal::Kitty => write!(f, "Kitty"),
            Terminal::Vscode => write!(f, "VSCode"),
            Terminal::VSCodeInsiders => write!(f, "VSCode Insiders"),
            Terminal::Tabby => write!(f, "Tabby"),
            Terminal::Nova => write!(f, "Nova"),
            Terminal::WezTerm => write!(f, "Wezterm"),
            Terminal::JediTerm(_) => write!(f, "Jetbrains"),
            Terminal::GnomeTerminal => write!(f, "Gnome Terminal"),
            Terminal::Konsole => write!(f, "Konsole"),
            Terminal::Tilix => write!(f, "Tilix"),
            Terminal::XfceTerminal => write!(f, "Xfce Terminal"),
            Terminal::Terminator => write!(f, "Terminator"),
        }
    }
}

impl Terminal {
    pub fn parent_terminal() -> Option<Self> {
        match std::env::var("TERM_PROGRAM").ok().as_deref() {
            Some("iTerm.app") => Some(Terminal::Iterm),
            Some("Apple_Terminal") => Some(Terminal::TerminalApp),
            Some("Hyper") => Some(Terminal::Hyper),
            Some("vscode") => match std::env::var("TERM_PROGRAM_VERSION").ok().as_deref() {
                Some(v) if v.contains("insiders") => Some(Terminal::VSCodeInsiders),
                _ => Some(Terminal::Vscode),
            },
            Some("Tabby") => Some(Terminal::Tabby),
            Some("Nova") => Some(Terminal::Nova),
            Some("WezTerm") => Some(Terminal::WezTerm),
            _ => match std::env::var("__CFBundleIdentifier").ok().as_deref() {
                Some(v) if v.contains("com.jetbrains.") => Some(Terminal::JediTerm(v.into())),
                _ => None,
            },
        }
        // TODO(grant): Improve this for Linux, it currently is not very accurate
    }

    pub fn internal_id(&self) -> String {
        match self {
            Terminal::Iterm => "iterm".into(),
            Terminal::TerminalApp => "terminal".into(),
            Terminal::Hyper => "hyper".into(),
            Terminal::Alacritty => "alacritty".into(),
            Terminal::Kitty => "kitty".into(),
            Terminal::Vscode => "vscode".into(),
            Terminal::VSCodeInsiders => "vscode-insiders".into(),
            Terminal::Tabby => "tabby".into(),
            Terminal::Nova => "nova".into(),
            Terminal::WezTerm => "wezterm".into(),
            Terminal::JediTerm(name) => name
                .trim_start_matches("com.jetbrains.")
                .trim_start_matches("com.google.")
                .to_string(),
            Terminal::GnomeTerminal => "gnome-terminal".into(),
            Terminal::Konsole => "konsole".into(),
            Terminal::Tilix => "tilix".into(),
            Terminal::XfceTerminal => "xfce-terminal".into(),
            Terminal::Terminator => "terminator".into(),
        }
    }

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
            Terminal::Nova => String::from("com.panic.Nova"),
            Terminal::WezTerm => String::from("com.github.wez.wezterm"),
            Terminal::JediTerm(id) => id.to_string(),
            Terminal::GnomeTerminal => todo!(),
            Terminal::Konsole => todo!(),
            Terminal::Tilix => todo!(),
            Terminal::XfceTerminal => todo!(),
            Terminal::Terminator => todo!(),
        }
    }

    pub fn executable_name(&self) -> Option<Cow<'static, str>> {
        match self {
            Terminal::Vscode => Some("code".into()),
            Terminal::VSCodeInsiders => Some("code-insiders".into()),
            Terminal::Alacritty => Some("alacritty".into()),
            Terminal::Kitty => Some("kitty".into()),
            Terminal::GnomeTerminal => Some("gnome-terminal-server".into()),
            Terminal::Konsole => Some("konsole".into()),
            Terminal::Tilix => Some("tilix".into()),
            Terminal::XfceTerminal => Some("xfce4-terminal".into()),
            _ => None,
        }
    }

    pub fn wm_class(&self) -> Option<Cow<'static, str>> {
        match self {
            Terminal::Vscode => Some("Code".into()),
            Terminal::VSCodeInsiders => Some("Vscode-insiders".into()),
            Terminal::GnomeTerminal => Some("Gnome-terminal".into()),
            Terminal::Konsole => Some("konsole".into()),
            Terminal::Tilix => Some("Tilix".into()),
            Terminal::Alacritty => Some("Alacritty".into()),
            Terminal::Kitty => Some("kitty".into()),
            Terminal::XfceTerminal => Some("Xfce4-terminal".into()),
            Terminal::Terminator => Some("Terminator".into()),
            _ => None,
        }
    }

    pub fn is_input_dependant(&self) -> bool {
        matches!(
            self,
            Terminal::WezTerm | Terminal::Alacritty | Terminal::Kitty | Terminal::Nova | Terminal::JediTerm(_)
        )
    }

    pub fn is_jetbrains_terminal() -> bool {
        // Handles all official JetBrain IDEs + Android Studio
        matches!(std::env::var("TERMINAL_EMULATOR").ok(), Some(v) if v == "JetBrains-JediTerm")
    }
}
