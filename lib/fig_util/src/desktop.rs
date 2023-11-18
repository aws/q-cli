use std::process::Command;

use crate::{
    directories,
    manifest,
    system_info,
    Error,
};

pub struct LaunchArgs {
    /// Should we wait for the socket to continue execution
    pub wait_for_socket: bool,
    /// Should we open the dashboard right away
    ///
    /// Note that this won't open the dashboard if the app is already running
    pub open_dashboard: bool,
    /// Should we do the first update check or skip it
    pub immediate_update: bool,
    /// Print output to user
    pub verbose: bool,
}

impl Default for LaunchArgs {
    fn default() -> Self {
        Self {
            wait_for_socket: false,
            open_dashboard: false,
            immediate_update: true,
            verbose: false,
        }
    }
}

pub fn is_codewhisperer_desktop_running() -> bool {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            use appkit_nsworkspace_bindings::NSRunningApplication;
            use macos_utils::{
                NSArray,
                NSString,
            };
            use objc::{
                class,
                msg_send,
                sel,
                sel_impl,
            };
            use sysinfo::{
                ProcessRefreshKind,
                RefreshKind,
                System,
                SystemExt,
            };

            use crate::consts::{
                CODEWHISPERER_DESKTOP_PROCESS_NAME,
                CODEWHISPERER_BUNDLE_ID
            };

            let bundle_id = NSString::from(CODEWHISPERER_BUNDLE_ID);
            let running_applications: NSArray<NSRunningApplication> = unsafe {
                msg_send![
                    class!(NSRunningApplication),
                    runningApplicationsWithBundleIdentifier: bundle_id
                ]
            };

            if !running_applications.is_empty() {
                return true;
            }

            // Fallback to process name check
            let s = System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()));
            let mut processes = s.processes_by_exact_name(CODEWHISPERER_DESKTOP_PROCESS_NAME);
            processes.next().is_some()
        } else if #[cfg(target_os = "windows")] {
            use crate::consts::CODEWHISPERER_DESKTOP_PROCESS_NAME;

            let output = match std::process::Command::new("tasklist.exe")
                .args(["/NH", "/FI", "IMAGENAME eq fig_desktop.exe"])
                .output()
            {
                Ok(output) => output,
                Err(_) => return false,
            };

            match std::str::from_utf8(&output.stdout) {
                Ok(result) => result.contains(CODEWHISPERER_DESKTOP_PROCESS_NAME),
                Err(_) => false,
            }
        } else {
            use sysinfo::{
                ProcessRefreshKind,
                RefreshKind,
                System,
                SystemExt,
            };

            use crate::consts::CODEWHISPERER_DESKTOP_PROCESS_NAME;

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
                        Ok(result) => result.contains(CODEWHISPERER_DESKTOP_PROCESS_NAME),
                        Err(_) => false,
                    };
                },
                false => "fig_desktop",
            };

            let s = System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()));
            let mut processes = s.processes_by_exact_name(process_name);
            processes.next().is_some()
        }
    }
}

pub fn launch_fig_desktop(args: LaunchArgs) -> Result<(), Error> {
    if manifest::is_headless() {
        return Err(Error::LaunchError(
            "launching Fig from headless installs is not yet supported".to_owned(),
        ));
    }

    if system_info::is_remote() {
        return Err(Error::LaunchError(
            "launching CodeWhisperer from remote installs is not yet supported".to_owned(),
        ));
    }

    match is_codewhisperer_desktop_running() {
        true => return Ok(()),
        false => {
            if args.verbose {
                println!("Launching CodeWhisperer...");
            }
        },
    }

    std::fs::remove_file(directories::desktop_socket_path()?).ok();

    let mut common_args = vec![];
    if !args.open_dashboard {
        common_args.push("--no-dashboard");
    }
    if !args.immediate_update {
        common_args.push("--ignore-immediate-update");
    }

    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            let output = Command::new("open")
                .args(["-g", "-b", crate::consts::CODEWHISPERER_BUNDLE_ID, "--args"])
                .args(common_args)
                .output()?;

            if !output.status.success() {
                return Err(Error::LaunchError(String::from_utf8_lossy(&output.stderr).to_string()))
            }
        } else if #[cfg(windows)] {
            use std::os::windows::process::CommandExt;
            use windows::Win32::System::Threading::DETACHED_PROCESS;

            Command::new("fig_desktop")
                .creation_flags(DETACHED_PROCESS.0)
                .spawn()?;
        } else {
            if system_info::in_wsl() {
                let output = Command::new(crate::consts::CODEWHISPERER_DESKTOP_PROCESS_NAME)
                    .output()?;

                if !output.status.success() {
                    return Err(Error::LaunchError(String::from_utf8_lossy(&output.stderr).to_string()))
                }
            } else {
                let output = Command::new("systemctl")
                    .args(["--user", "start", "codewhisperer"])
                    .output()?;

                if !output.status.success() {
                    return Err(Error::LaunchError(String::from_utf8_lossy(&output.stderr).to_string()))
                }
            }
        }
    }

    if !args.wait_for_socket {
        return Ok(());
    }

    if !is_codewhisperer_desktop_running() {
        return Err(Error::LaunchError("fig was unable launch successfully".to_owned()));
    }

    // Wait for socket to exist
    let path = directories::desktop_socket_path()?;

    cfg_if::cfg_if! {
        if #[cfg(windows)] {
            for _ in 0..30 {
                match path.metadata() {
                    Ok(_) => return Ok(()),
                    Err(err) => if let Some(code) = err.raw_os_error() {
                        // Windows can't query socket file existence
                        // Check against arbitrary error code
                        if code == 1920 {
                            return Ok(())
                        }
                    },
                }

                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        } else {
            for _ in 0..30 {
                // Wait for socket to exist
                if path.exists() {
                    return Ok(());
                }

                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }
    }

    Err(Error::LaunchError("failed to connect to socket".to_owned()))
}
