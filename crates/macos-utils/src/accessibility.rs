use accessibility_sys::AXIsProcessTrusted;

use crate::NSURL;
use crate::util::IdRef;

static ACCESSIBILITY_SETTINGS_URL: &str =
    "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility";

pub fn open_accessibility() {
    unsafe {
        let url = NSURL::from(ACCESSIBILITY_SETTINGS_URL);
        let shared: IdRef = msg_send![class!(NSWorkspace), sharedWorkspace];
        let _: () = msg_send![*shared, openURL: url];
    }
}

pub fn accessibility_is_enabled() -> bool {
    unsafe { AXIsProcessTrusted() }
}
