pub mod directories;
pub mod manifest;
mod open;
pub mod process_info;
mod shell;
pub mod system_info;
pub mod terminal;

#[cfg(target_os = "macos")]
pub mod launchd_plist;

use std::path::{
    Path,
    PathBuf,
};
use std::process::Command;

use cfg_if::cfg_if;
pub use open::{
    open_url,
    open_url_async,
};
pub use process_info::get_parent_process_exe;
use rand::Rng;
pub use shell::Shell;
pub use terminal::Terminal;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io operation error")]
    IoError(#[from] std::io::Error),
    #[error("unsupported platform")]
    UnsupportedPlatform,
    #[error("unsupported architecture")]
    UnsupportedArch,
    #[error(transparent)]
    Directory(#[from] crate::directories::DirectoryError),
    #[error("process has no parent")]
    NoParentProcess,
    #[error("could not find the os hwid")]
    HwidNotFound,
    #[error("the shell, `{0}`, isn't supported yet")]
    UnknownShell(String),
    #[error("missing environment variable `{0}`")]
    MissingEnv(&'static str),
    #[error("unknown display server `{0}`")]
    UnknownDisplayServer(String),
    #[error("unknown desktop `{0}`")]
    UnknownDesktop(String),
    #[error("failed to launch fig: `{0}`")]
    LaunchError(String),
}

pub fn gen_hex_string() -> String {
    let mut buf = [0u8; 32];
    rand::thread_rng().fill(&mut buf);
    hex::encode(buf)
}

pub fn search_xdg_data_dirs(ext: impl AsRef<std::path::Path>) -> Option<PathBuf> {
    let ext = ext.as_ref();
    if let Ok(xdg_data_dirs) = std::env::var("XDG_DATA_DIRS") {
        for base in xdg_data_dirs.split(':') {
            let check = Path::new(base).join(ext);
            if check.exists() {
                return Some(check);
            }
        }
    }
    None
}

pub fn is_app_running() -> bool {
    cfg_if! {
        if #[cfg(target_os = "macos")] {
            let output = match std::process::Command::new("lsappinfo")
                .args(["info", "-app", "com.mschrage.fig"])
                .output()
            {
                Ok(output) => output,
                Err(_) => return false,
            };

            match std::str::from_utf8(&output.stdout) {
                Ok(result) => !result.trim().is_empty(),
                Err(_) => false,
            }
        } else {
            cfg_if! {
                if #[cfg(windows)] {
                    let output = match std::process::Command::new("tasklist.exe").args(["/NH", "/FI", "IMAGENAME eq fig_desktop.exe"]).output() {
                        Ok(output) => output,
                        Err(_) => return false,
                    };

                    match std::str::from_utf8(&output.stdout) {
                        Ok(result) => result.contains("fig_desktop.exe"),
                        Err(_) => false,
                    }
                } else if #[cfg(unix)] {
                    use sysinfo::{
                        ProcessRefreshKind,
                        RefreshKind,
                        System,
                        SystemExt,
                    };

                    let process_name = match system_info::in_wsl() {
                        true => {
                            let output = match std::process::Command::new("tasklist.exe").args(["/NH", "/FI", "IMAGENAME eq fig_desktop.exe"]).output() {
                                Ok(output) => output,
                                Err(_) => return false,
                            };

                            return match std::str::from_utf8(&output.stdout) {
                                Ok(result) => result.contains("fig_desktop.exe"),
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
    }
}

pub fn launch_fig(wait_for_socket: bool, verbose: bool) -> Result<(), Error> {
    use directories::fig_socket_path;

    if system_info::is_remote() {
        return Err(Error::LaunchError(
            "launching Fig from remote installs is not yet supported".to_owned(),
        ));
    }

    match is_app_running() {
        true => return Ok(()),
        false => {
            if verbose {
                println!("Launching Fig...")
            }
        },
    }

    std::fs::remove_file(fig_socket_path()?).ok();

    cfg_if! {
        if #[cfg(unix)] {
            cfg_if! {
                if #[cfg(target_os = "macos")] {
                    let output = Command::new("open")
                        .args(["-g", "-b", "com.mschrage.fig", "--args", "--no-dashboard"])
                        .output()?;

                    if !output.status.success() {
                        return Err(Error::LaunchError(String::from_utf8_lossy(&output.stderr).to_string()))
                    }
                } else {
                    if system_info::in_wsl() {
                        let output = Command::new("fig_desktop.exe")
                            .output()?;

                        if !output.status.success() {
                            return Err(Error::LaunchError(String::from_utf8_lossy(&output.stderr).to_string()))
                        }
                    } else {
                        let output = Command::new("systemctl")
                            .args(&["--user", "start", "fig"])
                            .output()?;

                        if !output.status.success() {
                            return Err(Error::LaunchError(String::from_utf8_lossy(&output.stderr).to_string()))
                        }
                    }
                }
            }
        } else if #[cfg(windows)] {
            use std::os::windows::process::CommandExt;
            use windows::Win32::System::Threading::DETACHED_PROCESS;

            Command::new("fig_desktop")
                .creation_flags(DETACHED_PROCESS.0)
                .spawn()?;
        }
    }

    if !wait_for_socket {
        return Ok(());
    }

    if !is_app_running() {
        return Err(Error::LaunchError("fig was unable launch successfully".to_owned()));
    }

    // Wait for socket to exist
    let path = fig_socket_path()?;

    cfg_if! {
        if #[cfg(windows)] {
            for _ in 0..20 {
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
            for _ in 0..10 {
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
