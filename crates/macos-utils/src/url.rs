use std::path::{
    Path,
    PathBuf,
};

use appkit_nsworkspace_bindings::{
    INSURL,
    INSWorkspace,
    NSWorkspace,
};

use crate::{
    NSString,
    NSStringRef,
};

pub fn path_for_application(bundle_identifier: &str) -> Option<PathBuf> {
    let bundle_identifier: NSString = bundle_identifier.into();
    let url = unsafe {
        NSWorkspace::sharedWorkspace().URLForApplicationWithBundleIdentifier_(bundle_identifier.to_appkit_nsstring())
    };
    let path = unsafe { url.path() };
    let reference = unsafe { NSStringRef::new(path.0) };
    reference.as_str().map(|x| Path::new(x).to_path_buf())
}
