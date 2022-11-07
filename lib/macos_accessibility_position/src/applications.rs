use appkit_nsworkspace_bindings::{
    INSRunningApplication,
    INSWorkspace,
    NSRunningApplication,
    NSWorkspace,
    NSWorkspace_NSWorkspaceRunningApplications,
};

use crate::{
    NSArrayRef,
    NSStringRef,
};

#[derive(Debug)]
pub struct MacOSApplication {
    pub name: Option<String>,
    pub bundle_identifier: Option<String>,
    pub process_identifier: i32,
}

pub fn running_applications() -> Vec<MacOSApplication> {
    unsafe {
        let workspace = NSWorkspace::sharedWorkspace();

        let apps: NSArrayRef<NSRunningApplication> = workspace.runningApplications().into();
        apps.into_iter()
            .map(|app| {
                let application = NSRunningApplication(*app as *mut _);

                let name = NSStringRef::new(application.localizedName().0);
                let bundle_id = NSStringRef::new(application.bundleIdentifier().0);

                MacOSApplication {
                    name: name.as_str().map(|s| s.to_string()),
                    bundle_identifier: bundle_id.as_str().map(|s| s.to_string()),
                    process_identifier: application.processIdentifier(),
                }
            })
            .collect()
    }
}

pub fn running_applications_matching(bundle_identifier: &str) -> Vec<MacOSApplication> {
    running_applications()
        .into_iter()
        .filter_map(|app| {
            // todo: use `and_then` for more functional approach
            if matches!(&app.bundle_identifier, Some(bundle_id) if bundle_id.as_str() == bundle_identifier) {
                return Some(app);
            }

            None
        })
        .collect()
}

// #[test]
// fn test() {
//     let out = dbg!(running_applications());
// }
