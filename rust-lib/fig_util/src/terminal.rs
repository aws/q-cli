use std::fmt;

use once_cell::sync::Lazy;
use serde::{
    Deserialize,
    Serialize,
};

/// Terminals that macOS supports
pub const MACOS_TERMINALS: &[Terminal] = &[];

/// Terminals that Linux supports
pub const LINUX_TERMINALS: &[Terminal] = &[
    Terminal::Alacritty,
    Terminal::Kitty,
    Terminal::GnomeTerminal,
    Terminal::Konsole,
    Terminal::XfceTerminal,
    Terminal::WezTerm,
    Terminal::Tilix,
    Terminal::Terminator,
    Terminal::Vscode,
    Terminal::VSCodeInsiders,
];

/// Other terminals that figterm should launch within that are not full terminal emulators
pub const SPECIAL_TERMINALS: &[Terminal] = &[Terminal::Ssh, Terminal::Tmux, Terminal::Nvim];

pub static CURRENT_TERMINAL: Lazy<Option<Terminal>> = Lazy::new(Terminal::parent_terminal);

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
    /// Terminology
    Terminology,

    // Other pseudoterminal that we want to launch within
    /// SSH
    Ssh,
    /// Tmux
    Tmux,
    /// Nvim
    Nvim,
    /// Zellij
    Zellij,
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
            Terminal::Terminology => write!(f, "Terminology"),
            Terminal::Ssh => write!(f, "SSH"),
            Terminal::Tmux => write!(f, "Tmux"),
            Terminal::Nvim => write!(f, "Nvim"),
            Terminal::Zellij => write!(f, "Zellij"),
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
            Terminal::Terminology => "terminology".into(),
            Terminal::Ssh => "ssh".into(),
            Terminal::Tmux => "tmux".into(),
            Terminal::Nvim => "nvim".into(),
            Terminal::Zellij => "zellij".into(),
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
            _ => todo!(),
        }
    }

    pub fn executable_names(&self) -> &'static [&'static str] {
        match self {
            Terminal::Vscode => &["code"],
            Terminal::VSCodeInsiders => &["code-insiders"],
            Terminal::Alacritty => &["alacritty"],
            Terminal::Kitty => &["kitty"],
            Terminal::GnomeTerminal => &["gnome-terminal-server"],
            Terminal::Konsole => &["konsole"],
            Terminal::Tilix => &["tilix"],
            Terminal::XfceTerminal => &["xfce4-terminal"],
            Terminal::Terminology => &["terminology"],
            Terminal::WezTerm => &["wezterm", "wezterm-gui"],

            Terminal::Ssh => &["sshd"],

            Terminal::Tmux => &["tmux"],
            Terminal::Nvim => &["nvim"],
            Terminal::Zellij => &["zellij"],

            _ => &[],
        }
    }

    pub fn wm_class(&self) -> Option<&'static str> {
        match self {
            Terminal::Vscode => Some("Code"),
            Terminal::VSCodeInsiders => Some("Vscode-insiders"),
            Terminal::GnomeTerminal => Some("Gnome-terminal"),
            Terminal::Konsole => Some("konsole"),
            Terminal::Tilix => Some("Tilix"),
            Terminal::Alacritty => Some("Alacritty"),
            Terminal::Kitty => Some("kitty"),
            Terminal::XfceTerminal => Some("Xfce4-terminal"),
            Terminal::Terminator => Some("Terminator"),
            Terminal::Terminology => Some("terminology"),
            Terminal::WezTerm => Some("org.wezfurlong.wezterm"),
            _ => None,
        }
    }

    // corresponds to GSE source type
    pub fn gnome_id(&self) -> Option<&'static str> {
        match self {
            // Terminal::Vscode => Some("Code"),
            // Terminal::VSCodeInsiders => Some("Code - Insiders"),
            Terminal::GnomeTerminal => Some("gnome-terminal-server"),
            // Terminal::Konsole => Some("org.kde.konsole"),
            Terminal::Tilix => Some("tilix"),
            Terminal::Alacritty => Some("Alacritty"),
            // Terminal::Kitty => Some("kitty"),
            Terminal::XfceTerminal => Some("xfce4-terminal"),
            // Terminal::Terminator => Some("terminator"),
            // Terminal::Terminology => Some("terminology"),
            // Terminal::WezTerm => Some("org.wezfurlong.wezterm"),
            _ => None,
        }
    }

    /// (macos) do we need input method
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

    pub fn supports_fancy_boxes(&self) -> bool {
        !matches!(self, Terminal::Vscode | Terminal::VSCodeInsiders)
    }
}
