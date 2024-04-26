pub const APP_BUNDLE_ID: &str = "com.amazon.codewhisperer";
pub const APP_BUNDLE_NAME: &str = "Q.app";

#[cfg(not(target_os = "windows"))]
pub const APP_PROCESS_NAME: &str = "q_desktop";
#[cfg(target_os = "windows")]
pub const APP_PROCESS_NAME: &str = "q_desktop.exe";

pub const CLI_BINARY_NAME: &str = "q";
pub const CLI_BINARY_NAME_MINIMAL: &str = "q-minimal";
pub const PTY_BINARY_NAME: &str = "qterm";

pub const CLI_CRATE_NAME: &str = "q_cli";

pub const URL_SCHEMA: &str = "q";

pub const PRODUCT_NAME: &str = "Q";
pub const PRODUCT_NAME_SHORT: &str = "Q";

pub const RUNTIME_DIR_NAME: &str = "cwrun";

// These are the old "CodeWhisperer" branding, used anywhere we will not update to Q
pub const OLD_PRODUCT_NAME: &str = "CodeWhisperer";
pub const OLD_CLI_BINARY_NAME: &str = "cw";
pub const OLD_PTY_BINARY_NAME: &str = "cwterm";

pub const OLD_CLI_BINARY_NAMES: &[&str] = &["fig", "cw"];
pub const OLD_PTY_BINARY_NAMES: &[&str] = &["figterm", "cwterm"];

pub const GITHUB_DISCUSSIONS_REPO_NAME: &str = "codewhisperer-command-line-discussions";

pub mod url {
    pub const USER_MANUAL: &str = "https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/command-line.html";
    pub const AUTOCOMPLETE_WIKI: &str =
        "https://docs.aws.amazon.com/codewhisperer/latest/userguide/command-line-autocomplete.html";
    pub const AUTOCOMPLETE_SSH_WIKI: &str =
        "https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/command-line-ssh.html";
    pub const CHAT_WIKI: &str = "https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/command-line-chat.html";
    pub const TRANSLATE_WIKI: &str =
        "https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/command-line-conversation.html";
    pub const TELEMETRY_WIKI: &str = "https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/sharing-data.html";
}

/// macOS specific constants
pub mod macos {
    pub const BUNDLE_CONTENTS_MACOS_PATH: &str = "Contents/MacOS";
    pub const BUNDLE_CONTENTS_RESOURCE_PATH: &str = "Contents/Resources";
    pub const BUNDLE_CONTENTS_HELPERS_PATH: &str = "Contents/Helpers";
    pub const BUNDLE_CONTENTS_INFO_PLIST_PATH: &str = "Contents/Info.plist";
}

pub mod env_var {
    macro_rules! define_env_vars {
        ($($(#[$meta:meta])* $ident:ident = $name:expr),*) => {
            $(
                $(#[$meta])*
                pub const $ident: &str = $name;
            )*

            pub const ALL: &[&str] = &[$($ident),*];
        }
    }

    define_env_vars! {
        /// The UUID of the current parent qterm instance
        QTERM_SESSION_ID = "QTERM_SESSION_ID",

        /// The current parent socket to connect to
        Q_PARENT = "Q_PARENT",

        /// Set the [`Q_PARENT`] parent socket to connect to
        Q_SET_PARENT = "Q_SET_PARENT",

        /// Guard for the [`Q_SET_PARENT`] check
        Q_SET_PARENT_CHECK = "Q_SET_PARENT_CHECK",

        /// Set if qterm is running, contains the version
        Q_TERM = "Q_TERM",

        /// Sets the current log level
        Q_LOG_LEVEL = "Q_LOG_LEVEL",

        /// Overrides the ZDOTDIR environment variable
        Q_ZDOTDIR = "Q_ZDOTDIR",

        /// Indicates a process was launched by Q
        PROCESS_LAUNCHED_BY_Q = "PROCESS_LAUNCHED_BY_Q",

        /// The shell to use in qterm
        Q_SHELL = "Q_SHELL"
    }
}
