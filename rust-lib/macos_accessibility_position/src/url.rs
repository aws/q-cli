use appkit_nsworkspace_bindings::{
    INSWorkspace,
    NSWorkspace,
    INSURL,
};

use crate::{
    NSString,
    NSStringRef,
};

pub fn url_for_application(bundle_identifier: &str) -> Option<String> {
    let bundle_identifier: NSString = bundle_identifier.into();
    let url = unsafe {
        NSWorkspace::sharedWorkspace()
            .URLForApplicationWithBundleIdentifier_(bundle_identifier.to_appkit_nsstring())
            .absoluteString()
    };
    let reference = unsafe { NSStringRef::new(url.0) };
    reference.as_str().map(|x| x.to_string())
}
