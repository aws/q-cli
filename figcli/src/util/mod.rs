use std::{
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use sysinfo::{get_current_pid, ProcessExt, System, SystemExt};

pub mod backoff;
pub mod checksum;
pub mod shell;
pub mod sync;
pub mod terminal;

pub fn get_parent_process_exe() -> Result<PathBuf> {
    let mut system = System::new();
    let current_pid =
        get_current_pid().map_err(|_| anyhow::anyhow!("Could not get current pid"))?;
    if !system.refresh_process(current_pid) {
        anyhow::bail!("Could not find current process info")
    }
    let current_process = system
        .process(current_pid)
        .context("Could not find current process info")?;

    let parent_pid = current_process
        .parent()
        .context("Could not get parent pid")?;

    if !system.refresh_process(parent_pid) {
        anyhow::bail!("Could not find parent process info")
    }
    let parent_process = system
        .process(parent_pid)
        .context("Could not find parent process info")?;

    Ok(parent_process.exe().to_path_buf())
}

pub fn fig_bundle() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        Some(PathBuf::from("/Applications/Fig.app/"))
    }
    #[cfg(not(any(target_os = "macos")))]
    {
        None
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

#[cfg(target_os = "macos")]
pub fn app_path_from_bundle_id(bundle_id: impl AsRef<OsStr>) -> Option<String> {
    let installed_apps = Command::new("mdfind")
        .arg("kMDItemCFBundleIdentifier")
        .arg("=")
        .arg(bundle_id)
        .output()
        .ok()?;
    let path = String::from_utf8_lossy(&installed_apps.stdout);
    Some(path.trim().split('\n').next()?.into())
}

#[cfg(not(any(target_os = "macos")))]
pub fn app_path_from_bundle_id(_bundle_id: impl AsRef<OsStr>) -> Option<String> {
    None
}

pub fn get_shell() -> Result<String> {
    let ppid = nix::unistd::getppid();

    let result = Command::new("ps")
        .arg("-p")
        .arg(format!("{}", ppid))
        .arg("-o")
        .arg("comm=")
        .output()
        .context("Could not read value")?;

    Ok(String::from_utf8_lossy(&result.stdout).trim().into())
}

#[cfg(target_os = "macos")]
pub fn get_machine_id() -> Option<String> {
    let output = Command::new("ioreg")
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
}

#[cfg(target_os = "linux")]
pub fn get_machine_id() -> Option<String> {
    // https://man7.org/linux/man-pages/man5/machine-id.5.html
    std::fs::read_to_string("/var/lib/dbus/machine-id").ok()
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn get_machine_id() -> Option<String> {
    unimplemented!();
}

#[cfg(target_os = "macos")]
pub fn is_app_running() -> bool {
    let output = match Command::new("lsappinfo")
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
}

#[cfg(not(target_os = "macos"))]
pub fn is_app_running() -> bool {
    false
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
        Self {
            verbose: true,
            ..self
        }
    }
}

#[cfg(target_os = "macos")]
pub fn launch_fig(opts: LaunchOptions) -> Result<()> {
    use fig_ipc::get_fig_socket_path;

    if is_app_running() {
        return Ok(());
    }

    if opts.verbose {
        println!("\n→ Launching Fig...\n");
    }

    Command::new("open")
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
    anyhow::bail!("\nUnable to finish launching Fig properly\n")
}

#[cfg(not(any(target_os = "macos")))]
pub fn launch_fig(_opts: LaunchOptions) -> Result<()> {
    anyhow::bail!("Fig desktop can not be launched on this platform")
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
