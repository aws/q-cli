use accessibility_sys::AXIsProcessTrusted;
use appkit_nsworkspace_bindings::{
    INSWorkspace,
    NSWorkspace,
    INSURL,
    NSURL,
};

use super::util::NSString;

static ACCESSIBILITY_SETTINGS_URL: &str =
    "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility";

pub fn open_accessibility() {
    unsafe {
        let url_str: NSString = ACCESSIBILITY_SETTINGS_URL.into();
        let url = NSURL::alloc();
        let res = url.initWithString_(url_str.into());
        NSWorkspace::sharedWorkspace().openURL_(NSURL(res));
    }
}

pub fn accessibility_is_enabled() -> bool {
    unsafe { AXIsProcessTrusted() }
}
