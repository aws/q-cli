use std::fmt;

use once_cell::sync::Lazy;
use serde::{
    Deserialize,
    Serialize,
};

/// Terminals that macOS supports
pub const MACOS_TERMINALS: &[Terminal] = &[
    Terminal::Alacritty,
    Terminal::Iterm,
    Terminal::Kitty,
    Terminal::Tabby,
    Terminal::TerminalApp,
    Terminal::VSCodeInsiders,
    Terminal::VSCode,
    Terminal::VSCodium,
    Terminal::WezTerm,
];

/// Terminals that Linux supports
pub const LINUX_TERMINALS: &[Terminal] = &[
    Terminal::Alacritty,
    Terminal::Kitty,
    Terminal::GnomeConsole,
    Terminal::GnomeTerminal,
    Terminal::Hyper,
    Terminal::Konsole,
    Terminal::XfceTerminal,
    Terminal::WezTerm,
    Terminal::Tilix,
    Terminal::Terminator,
    Terminal::VSCode,
    Terminal::VSCodeInsiders,
    Terminal::VSCodium,
    Terminal::IntelliJ(None),
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
    VSCode,
    /// VSCode Insiders
    VSCodeInsiders,
    /// VSCodium
    VSCodium,
    /// Tabby
    Tabby,
    /// Nova
    Nova,
    /// Wezterm
    WezTerm,
    /// Gnome Console
    GnomeConsole,
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
    /// IntelliJ
    IntelliJ(Option<IntelliJVariant>),

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
            Terminal::VSCode => write!(f, "VSCode"),
            Terminal::VSCodeInsiders => write!(f, "VSCode Insiders"),
            Terminal::VSCodium => write!(f, "VSCodium"),
            Terminal::Tabby => write!(f, "Tabby"),
            Terminal::Nova => write!(f, "Nova"),
            Terminal::WezTerm => write!(f, "Wezterm"),
            Terminal::GnomeConsole => write!(f, "Gnome Console"),
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
            Terminal::IntelliJ(Some(variant)) => write!(f, "{}", variant.application_name()),
            Terminal::IntelliJ(None) => write!(f, "IntelliJ"),
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
                _ => Some(Terminal::VSCode),
            },
            Some("Tabby") => Some(Terminal::Tabby),
            Some("Nova") => Some(Terminal::Nova),
            Some("WezTerm") => Some(Terminal::WezTerm),
            _ => match std::env::var("__CFBundleIdentifier").ok().as_deref() {
                Some(v) => Self::from_bundle_id(v),
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
            Terminal::VSCode => "vscode".into(),
            Terminal::VSCodeInsiders => "vscode-insiders".into(),
            Terminal::VSCodium => "vscodium".into(),
            Terminal::Tabby => "tabby".into(),
            Terminal::Nova => "nova".into(),
            Terminal::WezTerm => "wezterm".into(),
            Terminal::GnomeTerminal => "gnome-terminal".into(),
            Terminal::GnomeConsole => "gnome-console".into(),
            Terminal::Konsole => "konsole".into(),
            Terminal::Tilix => "tilix".into(),
            Terminal::XfceTerminal => "xfce-terminal".into(),
            Terminal::Terminator => "terminator".into(),
            Terminal::Terminology => "terminology".into(),
            Terminal::Ssh => "ssh".into(),
            Terminal::Tmux => "tmux".into(),
            Terminal::Nvim => "nvim".into(),
            Terminal::Zellij => "zellij".into(),
            Terminal::IntelliJ(ide) => match ide {
                Some(variant) => format!("intellij-{}", variant.internal_id()),
                None => "intellij".into(),
            },
        }
    }

    /// Get the bundle identifier for the terminal
    /// Note: this does not gracefully handle terminals that have changed bundle identifiers
    /// recently such as VSCodium & Alacritty. We default to the current identifier.
    pub fn to_bundle_id(&self) -> String {
        match self {
            Terminal::Iterm => String::from("com.googlecode.iterm2"),
            Terminal::TerminalApp => String::from("com.apple.Terminal"),
            Terminal::Hyper => String::from("co.zeit.hyper"),
            Terminal::Alacritty => String::from("org.alacritty"),
            Terminal::Kitty => String::from("net.kovidgoyal.kitty"),
            Terminal::VSCode => String::from("com.microsoft.VSCode"),
            Terminal::VSCodeInsiders => String::from("com.microsoft.VSCodeInsiders"),
            Terminal::VSCodium => String::from("com.vscodium"),
            Terminal::Tabby => String::from("org.tabby"),
            Terminal::Nova => String::from("com.panic.Nova"),
            Terminal::WezTerm => String::from("com.github.wez.wezterm"),
            Terminal::IntelliJ(Some(variant)) => variant.bundle_identifier().into(),
            _ => todo!(),
        }
    }

    pub fn from_bundle_id(bundle: impl AsRef<str>) -> Option<Self> {
        let bundle = bundle.as_ref();
        let res = match bundle {
            "com.googlecode.iterm2" => Terminal::Iterm,
            "com.apple.Terminal" => Terminal::TerminalApp,
            "co.zeit.hyper" => Terminal::Hyper,
            "io.alacritty" | "org.alacritty" => Terminal::Alacritty,
            "net.kovidgoyal.kitty" => Terminal::Kitty,
            "com.microsoft.VSCode" => Terminal::VSCode,
            "com.microsoft.VSCodeInsiders" => Terminal::VSCodeInsiders,
            "com.vscodium" | "com.visualstudio.code.oss" => Terminal::VSCodium,
            "org.tabby" => Terminal::Tabby,
            "com.panic.Nova" => Terminal::Nova,
            "com.github.wez.wezterm" => Terminal::WezTerm,
            // todo(mschrage): the following line does not account for Android Studio
            _ if bundle.starts_with("com.jetbrains.") | bundle.starts_with("com.google.") => {
                Terminal::IntelliJ(IntelliJVariant::from_bundle_id(bundle))
            },
            _ => return None,
        };

        Some(res)
    }

    pub fn supports_macos_input_method(&self) -> bool {
        matches!(
            self,
            Terminal::Alacritty | Terminal::Kitty | Terminal::Nova | Terminal::WezTerm | Terminal::IntelliJ(_)
        )
    }

    pub fn supports_macos_accessibility(&self) -> bool {
        matches!(
            self,
            Terminal::Iterm
                | Terminal::TerminalApp
                | Terminal::VSCode
                | Terminal::VSCodeInsiders
                | Terminal::VSCodium
                | Terminal::Hyper
                | Terminal::Tabby
        )
    }

    pub fn is_xterm(&self) -> bool {
        matches!(
            self,
            Terminal::VSCode | Terminal::VSCodeInsiders | Terminal::Hyper | Terminal::Tabby
        )
    }

    pub fn executable_names(&self) -> &'static [&'static str] {
        match self {
            Terminal::VSCode => &["code"],
            Terminal::VSCodeInsiders => &["code-insiders"],
            Terminal::Alacritty => &["alacritty"],
            Terminal::Kitty => &["kitty"],
            Terminal::GnomeConsole => &["kgx"],
            Terminal::GnomeTerminal => &["gnome-terminal-server"],
            Terminal::Konsole => &["konsole"],
            Terminal::Tilix => &["tilix"],
            Terminal::XfceTerminal => &["xfce4-terminal"],
            Terminal::Terminology => &["terminology"],
            Terminal::WezTerm => &["wezterm", "wezterm-gui"],
            Terminal::Hyper => &["hyper"],
            Terminal::Tabby => &["tabby"],
            Terminal::Terminator => &["terminator"],

            Terminal::Ssh => &["sshd"],
            Terminal::Tmux => &["tmux"],
            Terminal::Nvim => &["nvim"],
            Terminal::Zellij => &["zellij"],

            _ => &[],
        }
    }

    pub fn wm_class(&self) -> Option<&'static str> {
        match self {
            Terminal::VSCode => Some("Code"),
            Terminal::VSCodeInsiders => Some("Vscode-insiders"),
            Terminal::GnomeConsole => Some("Kgx"),
            Terminal::GnomeTerminal => Some("Gnome-terminal"),
            Terminal::Hyper => Some("Hyper"),
            Terminal::Konsole => Some("konsole"),
            Terminal::Tilix => Some("Tilix"),
            Terminal::Alacritty => Some("Alacritty"),
            Terminal::Kitty => Some("kitty"),
            Terminal::XfceTerminal => Some("Xfce4-terminal"),
            Terminal::Terminator => Some("Terminator"),
            Terminal::Terminology => Some("terminology"),
            Terminal::WezTerm => Some("org.wezfurlong.wezterm"),
            Terminal::Tabby => Some("tabby"),
            Terminal::IntelliJ(Some(IntelliJVariant::IdeaCE)) => Some("jetbrains-idea-ce"),
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
            Terminal::Kitty => Some("kitty"),
            Terminal::XfceTerminal => Some("xfce4-terminal"),
            Terminal::Terminator => Some("terminator"),
            // Terminal::Terminology => Some("terminology"),
            // Terminal::WezTerm => Some("org.wezfurlong.wezterm"),
            // Terminal::Tabby => Some("tabby"),
            _ => None,
        }
    }

    /// (macos) do we need input method
    pub fn is_input_dependant(&self) -> bool {
        matches!(
            self,
            Terminal::WezTerm | Terminal::Alacritty | Terminal::Kitty | Terminal::Nova | Terminal::IntelliJ(_)
        )
    }

    pub fn is_jetbrains_terminal() -> bool {
        // Handles all official JetBrain IDEs + Android Studio
        matches!(std::env::var("TERMINAL_EMULATOR").ok(), Some(v) if v == "JetBrains-JediTerm")
    }

    pub fn supports_fancy_boxes(&self) -> bool {
        !matches!(self, Terminal::VSCode | Terminal::VSCodeInsiders | Terminal::VSCodium)
    }

    pub fn positioning_kind(&self) -> PositioningKind {
        match self {
            Terminal::Konsole => PositioningKind::Logical,
            _ => PositioningKind::Physical,
        }
    }
}

#[derive(Debug)]
pub enum PositioningKind {
    Logical,
    Physical,
}

macro_rules! intellij_variants {
    ($($name:ident { org: $organization:expr, internal_id: $internal_id:expr, name: $application_name:expr, bundle: $bundle_identifier:expr },)*) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(rename_all = "kebab-case")]
        pub enum IntelliJVariant {
            $(
                $name,
            )*
        }

        impl IntelliJVariant {
            pub const fn all() -> &'static [IntelliJVariant] {
                &[$(IntelliJVariant::$name,)*]
            }

            pub fn application_name(&self) -> &'static str {
                match self {
                    $(
                        IntelliJVariant::$name => $application_name,
                    )*
                }
            }

            pub fn organization(&self) -> &'static str {
                match self {
                    $(
                        IntelliJVariant::$name => $organization,
                    )*
                }
            }

            pub fn bundle_identifier(&self) -> &'static str {
                match self {
                    $(
                        IntelliJVariant::$name => $bundle_identifier,
                    )*
                }
            }

            pub fn internal_id(&self) -> &'static str {
                match self {
                    $(
                        IntelliJVariant::$name => $internal_id,
                    )*
                }
            }

            pub fn from_bundle_id(bundle_id: &str) -> Option<IntelliJVariant> {
                match bundle_id {
                    $(
                        $bundle_identifier => Some(IntelliJVariant::$name),
                    )*
                    _ => None,
                }
            }
        }
    };
}

intellij_variants! {
    IdeaUltimate {
        org: "JetBrains",
        internal_id: "idea-ultimate",
        name: "IDEA Ultimate",
        bundle: "com.jetbrains.intellij"
    },
    IdeaCE {
        org: "JetBrains",
        internal_id: "idea-ce",
        name: "IDEA Community",
        bundle: "com.jetbrains.intellij.ce"
    },
    WebStorm {
        org: "JetBrains",
        internal_id: "webstorm",
        name: "WebStorm",
        bundle: "com.jetbrains.WebStorm"
    },
    GoLand {
        org: "JetBrains",
        internal_id: "goland",
        name: "GoLand",
        bundle: "com.jetbrains.goland"
    },
    PhpStorm {
        org: "JetBrains",
        internal_id: "phpstorm",
        name: "PhpStorm",
        bundle: "com.jetbrains.PhpStorm"
    },
    PyCharm {
        org: "JetBrains",
        internal_id: "pycharm",
        name: "PyCharm Professional",
        bundle: "com.jetbrains.pycharm"
    },
    PyCharmCE {
        org: "JetBrains",
        internal_id: "pycharm-ce",
        name: "PyCharm Community",
        bundle: "com.jetbrains.pycharm.ce"
    },
    AppCode {
        org: "JetBrains",
        internal_id: "appcode",
        name: "AppCode",
        bundle: "com.jetbrains.AppCode"
    },
    CLion {
        org: "JetBrains",
        internal_id: "clion",
        name: "CLion",
        bundle: "com.jetbrains.CLion"
    },
    Rider {
        org: "JetBrains",
        internal_id: "rider",
        name: "Rider",
        bundle: "com.jetbrains.rider"
    },
    RubyMine {
        org: "JetBrains",
        internal_id: "rubymine",
        name: "RubyMine",
        bundle: "com.jetbrains.rubymine"
    },
    DataSpell {
        org: "JetBrains",
        internal_id: "dataspell",
        name: "DataSpell",
        bundle: "com.jetbrains.dataspell"
    },
    AndroidStudio {
        org: "Google",
        internal_id: "android-studio",
        name: "Android Studio",
        bundle: "com.google.android.studio"
    },
}

impl IntelliJVariant {
    pub fn from_product_code(from: &str) -> Option<Self> {
        Some(match from {
            "IU" => IntelliJVariant::IdeaUltimate,
            "IC" => IntelliJVariant::IdeaCE,
            "PC" => IntelliJVariant::PyCharmCE,
            _ => return None,
        })
    }
}
