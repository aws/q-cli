pub const FIG_BUNDLE_ID: &str = "com.mschrage.fig";

#[cfg(not(target_os = "windows"))]
pub const FIG_DESKTOP_PROCESS_NAME: &str = "fig_desktop";
#[cfg(target_os = "windows")]
pub const FIG_DESKTOP_PROCESS_NAME: &str = "fig_desktop.exe";

#[cfg(target_os = "macos")]
pub const FIG_CLI_BINARY_NAME: &str = "fig-darwin-universal";
#[cfg(not(target_os = "macos"))]
pub const FIG_CLI_BINARY_NAME: &str = "fig";

pub const FIGTERM_BINARY_NAME: &str = "figterm";

pub const FIG_SCRIPTS_SCHEMA_VERSION: i64 = 3;
