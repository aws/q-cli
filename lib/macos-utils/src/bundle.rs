use std::path::PathBuf;

use core_foundation::base::TCFType;
use core_foundation::bundle::{
    CFBundleCopyBundleURL,
    CFBundleGetMainBundle,
};
use core_foundation::url::{
    CFURL,
    CFURLRef,
};

pub fn get_bundle_path() -> Option<PathBuf> {
    let url: CFURLRef = unsafe { CFBundleCopyBundleURL(CFBundleGetMainBundle()) };
    if url.is_null() {
        return None;
    }
    let url = unsafe { CFURL::wrap_under_get_rule(url) };
    url.to_path()
}

pub fn get_bundle_path_for_executable(executable: &str) -> Option<PathBuf> {
    get_bundle_path().and_then(|path| {
        let full_path = path.join("Contents").join("MacOS").join(executable);
        if full_path.exists() { Some(full_path) } else { None }
    })
}

pub fn get_bundle_resource_path() -> Option<PathBuf> {
    get_bundle_path().map(|path| path.join("Contents").join("Resources"))
}
