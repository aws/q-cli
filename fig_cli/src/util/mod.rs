pub mod api;
pub mod backoff;
pub mod checksum;
pub mod shell;
pub mod sync;

use std::env;
use std::ffi::OsStr;
use std::path::{
    Path,
    PathBuf,
};

use anyhow::{
    bail,
    Context,
    Result,
};
use cfg_if::cfg_if;
use fig_ipc::get_fig_socket_path;
use globset::{
    Glob,
    GlobSet,
    GlobSetBuilder,
};
use sysinfo::{
    get_current_pid,
    ProcessExt,
    ProcessRefreshKind,
    RefreshKind,
    System,
    SystemExt,
};

pub fn get_parent_process_exe() -> Result<PathBuf> {
    let mut system = System::new();
    let current_pid = get_current_pid().map_err(|_| anyhow::anyhow!("Could not get current pid"))?;
    if !system.refresh_process(current_pid) {
        anyhow::bail!("Could not find current process info")
    }
    let current_process = system
        .process(current_pid)
        .context("Could not find current process info")?;

    let parent_pid = current_process.parent().context("Could not get parent pid")?;

    if !system.refresh_process(parent_pid) {
        anyhow::bail!("Could not find parent process info")
    }
    let parent_process = system
        .process(parent_pid)
        .context("Could not find parent process info")?;

    Ok(parent_process.exe().to_path_buf())
}

#[must_use]
pub fn fig_bundle() -> Option<PathBuf> {
    cfg_if! {
        if #[cfg(target_os = "macos")] {
            Some(PathBuf::from("/Applications/Fig.app/"))
        } else {
            None
        }
    }
}

/// Glob patterns against full paths
pub fn glob_dir(glob: &GlobSet, directory: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    // List files in the directory
    let dir = std::fs::read_dir(directory)?;

    for entry in dir {
        let path = entry?.path();

        // Check if the file matches the glob pattern
        if glob.is_match(&path) {
            files.push(path);
        }
    }

    Ok(files)
}

/// Glob patterns agains the file name
pub fn glob_files(glob: &GlobSet, directory: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    // List files in the directory
    let dir = std::fs::read_dir(directory)?;

    for entry in dir {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name();

        // Check if the file matches the glob pattern
        if let Some(file_name) = file_name {
            if glob.is_match(file_name) {
                files.push(path);
            }
        }
    }

    Ok(files)
}

pub fn glob<I, S>(patterns: I) -> Result<GlobSet>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern.as_ref())?);
    }
    Ok(builder.build()?)
}

pub fn app_path_from_bundle_id(bundle_id: impl AsRef<OsStr>) -> Option<String> {
    cfg_if! {
        if #[cfg(target_os = "macos")] {
            let installed_apps = std::process::Command::new("mdfind")
                .arg("kMDItemCFBundleIdentifier")
                .arg("=")
                .arg(bundle_id)
                .output()
                .ok()?;

            let path = String::from_utf8_lossy(&installed_apps.stdout);
            Some(path.trim().split('\n').next()?.into())
        } else {
            let _bundle_id = bundle_id;
            None
        }
    }
}

#[must_use]
pub fn get_machine_id() -> Option<String> {
    cfg_if! {
        if #[cfg(target_os = "macos")] {
            let output = std::process::Command::new("ioreg")
                .args(&["-rd1", "-c", "IOPlatformExpertDevice"])
                .output()
                .ok()?;

            let output = String::from_utf8_lossy(&output.stdout);

            let machine_id = output
                .lines()
                .find(|line| line.contains("IOPlatformUUID"))?
                .split('=')
                .nth(1)?
                .trim()
                .trim_start_matches('"')
                .trim_end_matches('"')
                .into();

            Some(machine_id)
        } else if #[cfg(target_os = "linux")] {
            // https://man7.org/linux/man-pages/man5/machine-id.5.html
            std::fs::read_to_string("/var/lib/dbus/machine-id").ok()
        } else {
            None
        }
    }
}

#[must_use]
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
        } else if #[cfg(target_os = "linux")] {
            let s = System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()));
            let mut processes = s.processes_by_exact_name("fig_desktop");
            processes.next().is_some()
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[must_use]
pub struct LaunchOptions {
    pub wait_for_activation: bool,
    pub verbose: bool,
}

impl LaunchOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn wait_for_activation(self) -> Self {
        Self {
            wait_for_activation: true,
            ..self
        }
    }

    pub fn verbose(self) -> Self {
        Self { verbose: true, ..self }
    }
}

pub fn launch_fig(opts: LaunchOptions) -> Result<()> {
    cfg_if! {
        if #[cfg(target_os = "macos")] {
            if is_app_running() {
                return Ok(());
            }

            if opts.verbose {
                println!("\n→ Launching Fig...\n");
            }

            std::process::Command::new("open")
                .args(["-g", "-b", "com.mschrage.fig"])
                .output()
                .context("\nUnable to launch Fig\n")?;

            if !opts.wait_for_activation {
                return Ok(());
            }

            if !is_app_running() {
                anyhow::bail!("Unable to launch Fig");
            }

            // Wait for socket to exist
            let path = get_fig_socket_path();
            for _ in 0..9 {
                if path.exists() {
                    return Ok(());
                }
                // Sleep for a bit
                std::thread::sleep(std::time::Duration::from_millis(500));
            }

            bail!("\nUnable to finish launching Fig properly\n")
        } else if #[cfg(target_os = "linux")] {
            if is_app_running() {
                return Ok(());
            }

            if opts.verbose {
                println!("\n→ Launching Fig...\n");
            }

            std::fs::remove_file(get_fig_socket_path()).ok();

            let process = std::process::Command::new("systemctl")
                .args(&["--user", "start", "fig"])
                .output()
                .context("\nUnable to launch Fig\n")?;

            if !process.status.success() {
                bail!("Failed to launch fig.desktop");
            }


            if !opts.wait_for_activation {
                return Ok(());
            }

            if !is_app_running() {
                anyhow::bail!("Unable to launch Fig");
            }

            // Wait for socket to exist
            let path = get_fig_socket_path();
            for _ in 0..9 {
                if path.exists() {
                    return Ok(());
                }
                // Sleep for a bit
                std::thread::sleep(std::time::Duration::from_millis(500));
            }

            bail!("\nUnable to finish launching Fig properly\n")
        } else {
            let _opts = opts;
            bail!("Fig desktop can not be launched on this platform")
        }
    }
}

pub fn is_executable_in_path(program: impl AsRef<Path>) -> bool {
    match env::var_os("PATH") {
        Some(path) => env::split_paths(&path).any(|p| p.join(&program).is_file()),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_machine_id() {
        let machine_id = get_machine_id();
        assert!(machine_id.is_some());
    }
}
