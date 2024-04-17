pub const APP_BUNDLE_ID: &str = "com.amazon.codewhisperer";
pub const APP_BUNDLE_NAME: &str = "CodeWhisperer.app";

#[cfg(not(target_os = "windows"))]
pub const APP_PROCESS_NAME: &str = "codewhisperer_desktop";
#[cfg(target_os = "windows")]
pub const APP_PROCESS_NAME: &str = "codewhisperer_desktop.exe";

pub const CLI_BINARY_NAME: &str = "cw";
pub const PTY_BINARY_NAME: &str = "cwterm";

pub const URL_SCHEMA: &str = "codewhisperer";
