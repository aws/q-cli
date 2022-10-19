pub const FIG_BUNDLE_ID: &str = "com.mschrage.fig";

pub const FIG_DESKTOP_PROCESS_NAME: &str = "fig_desktop";
pub const FIG_DESKTOP_PROCESS_NAME_WINDOWS: &str = "fig_desktop.exe";

#[cfg(target_os = "macos")]
pub const FIG_CLI_BINARY_NAME: &str = "fig-darwin-universal";
#[cfg(not(target_os = "macos"))]
pub const FIG_CLI_BINARY_NAME: &str = "fig";
