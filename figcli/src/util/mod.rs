use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
};
use sysinfo::{get_current_pid, ProcessExt, System, SystemExt};

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
    unimplemented!();
}

/// Glob patterns against full paths
pub fn glob_dir(glob: &GlobSet, directory: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    // List files in the directory
    let dir = std::fs::read_dir(directory)?;

    for entry in dir {
        let entry = entry?;
        let path = entry.path();

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

#[cfg(not(any(target_os = "macos")))]
pub fn app_path_from_bundle_id(_bundle_id: impl AsRef<OsStr>) -> Option<String> {
    unimplemented!();
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
pub fn get_app_info() -> Result<String> {
    let output = Command::new("lsappinfo")
        .args(["info", "-app", "com.mschrage.fig"])
        .output()?;
    let result = String::from_utf8(output.stdout)?;
    Ok(result.trim().into())
}

#[cfg(target_os = "macos")]
pub fn is_app_running() -> bool {
    match get_app_info() {
        Ok(s) => !s.is_empty(),
        _ => false,
    }
}

#[cfg(target_os = "macos")]
pub fn launch_fig() -> Result<()> {
    if is_app_running() {
        return Ok(());
    }
    Command::new("open")
        .args(["-g", "-b", "com.mschrage.fig"])
        .spawn()
        .context("fig could not be launched")?;
    std::thread::sleep(std::time::Duration::from_secs(3));
    Ok(())
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
