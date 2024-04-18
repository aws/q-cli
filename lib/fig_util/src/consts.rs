pub const APP_BUNDLE_ID: &str = "com.amazon.codewhisperer";
pub const APP_BUNDLE_NAME: &str = "Q.app";

#[cfg(not(target_os = "windows"))]
pub const APP_PROCESS_NAME: &str = "q_desktop";
#[cfg(target_os = "windows")]
pub const APP_PROCESS_NAME: &str = "q_desktop.exe";

pub const CLI_BINARY_NAME: &str = "q";
pub const CLI_BINARY_NAME_MINIMAL: &str = "q-minimal";
pub const PTY_BINARY_NAME: &str = "qterm";

pub const URL_SCHEMA: &str = "q";

pub const PRODUCT_NAME: &str = "Q";
pub const PRODUCT_NAME_SHORT: &str = "Q";

pub const RUNTIME_DIR_NAME: &str = "cwrun";

// These are the old "CodeWhisperer" branding, used anywhere we will not update to Q
pub const OLD_PRODUCT_NAME: &str = "CodeWhisperer";
pub const OLD_CLI_BINARY_NAME: &str = "cw";

/// macOS specific constants
pub mod macos {
    pub const BUNDLE_CONTENTS_MACOS_PATH: &str = "Contents/MacOS";
    pub const BUNDLE_CONTENTS_RESOURCE_PATH: &str = "Contents/Resources";
    pub const BUNDLE_CONTENTS_HELPERS_PATH: &str = "Contents/Helpers";
    pub const BUNDLE_CONTENTS_INFO_PLIST_PATH: &str = "Contents/Info.plist";
}
