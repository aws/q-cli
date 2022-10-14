#[cfg(target_os = "macos")]
pub fn is_fig_desktop_running() -> bool {
    use appkit_nsworkspace_bindings::NSRunningApplication;
    use macos_accessibility_position::{
        NSArray,
        NSString,
    };
    use objc::{
        class,
        msg_send,
        sel,
        sel_impl,
    };

    use crate::consts::FIG_BUNDLE_ID;

    let bundle_id = NSString::from(FIG_BUNDLE_ID);

    let running_applications: NSArray<NSRunningApplication> = unsafe {
        msg_send![
            class!(NSRunningApplication),
            runningApplicationsWithBundleIdentifier: bundle_id
        ]
    };

    !running_applications.is_empty()
}

#[cfg(target_os = "windows")]
pub fn is_fig_desktop_running() -> bool {
    use crate::consts::FIG_DESKTOP_PROCESS_NAME;

    let output = match std::process::Command::new("tasklist.exe")
        .args(["/NH", "/FI", "IMAGENAME eq fig_desktop.exe"])
        .output()
    {
        Ok(output) => output,
        Err(_) => return false,
    };

    match std::str::from_utf8(&output.stdout) {
        Ok(result) => result.contains(FIG_DESKTOP_PROCESS_NAME),
        Err(_) => false,
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn is_fig_desktop_running() -> bool {
    use sysinfo::{
        ProcessRefreshKind,
        RefreshKind,
        System,
        SystemExt,
    };

    use crate::consts::FIG_DESKTOP_PROCESS_NAME;

    let process_name = match crate::system_info::in_wsl() {
        true => {
            let output = match std::process::Command::new("tasklist.exe")
                .args(["/NH", "/FI", "IMAGENAME eq fig_desktop.exe"])
                .output()
            {
                Ok(output) => output,
                Err(_) => return false,
            };

            return match std::str::from_utf8(&output.stdout) {
                Ok(result) => result.contains(FIG_DESKTOP_PROCESS_NAME),
                Err(_) => false,
            };
        },
        false => "fig_desktop",
    };

    let s = System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()));
    let mut processes = s.processes_by_exact_name(process_name);
    processes.next().is_some()
}
