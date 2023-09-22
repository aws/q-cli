pub const CODEWHISPERER_BUNDLE_ID: &str = "com.amazon.codewhisperer";

#[cfg(not(target_os = "windows"))]
pub const CODEWHISPERER_DESKTOP_PROCESS_NAME: &str = "codewhisperer_desktop";
#[cfg(target_os = "windows")]
pub const CODEWHISPERER_DESKTOP_PROCESS_NAME: &str = "codewhisperer_desktop.exe";

pub const CODEWHISPERER_CLI_BINARY_NAME: &str = "cw";

pub const CWTERM_BINARY_NAME: &str = "cwterm";

pub const FIG_SCRIPTS_SCHEMA_VERSION: i64 = 4;
