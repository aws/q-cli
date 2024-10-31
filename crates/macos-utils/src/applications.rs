use appkit_nsworkspace_bindings::{
    INSRunningApplication,
    INSURL,
    INSWorkspace,
    NSRunningApplication,
    NSURL,
    NSWorkspace,
    NSWorkspace_NSWorkspaceRunningApplications,
};

use crate::{
    NSArrayRef,
    NSString,
    NSStringRef,
};

#[derive(Debug)]
pub struct MacOSApplication {
    pub name: Option<String>,
    pub bundle_identifier: Option<String>,
    pub process_identifier: i32,
    pub bundle_path: Option<String>,
}

impl MacOSApplication {
    pub fn terminate(&self) {
        use nix::sys::signal::{
            self,
            Signal,
        };
        use nix::unistd::Pid;

        signal::kill(Pid::from_raw(self.process_identifier), Signal::SIGTERM).ok();
    }
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
                let bundle_path = NSStringRef::new(application.bundleURL().path().0);

                MacOSApplication {
                    name: name.as_str().map(|s| s.to_string()),
                    bundle_identifier: bundle_id.as_str().map(|s| s.to_string()),
                    bundle_path: bundle_path.as_str().map(|s| s.to_string()),
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

pub fn launch_application(bundle_path: &str) {
    unsafe {
        let workspace = NSWorkspace::sharedWorkspace();

        let str: NSString = bundle_path.into();
        let url = NSURL::fileURLWithPath_isDirectory_(str.to_appkit_nsstring(), objc::runtime::YES);

        workspace.openURL_(url);
    }
}

// #[test]
// fn test() {
//     // let out = dbg!(running_applications());
//     launch_application("/Applications/Alacritty.app")
// }

// #[test]
// fn test_terminate() {
//     // let out = dbg!(running_applications());
//     running_applications_matching("com.amazon.codewhisperercursor").into_iter().for_each(|app| {
//         println!("Terminating {}", app.process_identifier);
//         app.terminate()
//     })
// }
