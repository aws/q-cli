use anyhow::{Context, Result};
use directories::ProjectDirs;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
};

pub mod checksum;
pub mod settings;
pub mod shell;
pub mod terminal;

pub fn project_dir() -> Option<ProjectDirs> {
    directories::ProjectDirs::from("io", "fig", "fig")
}

pub fn home_dir() -> Result<PathBuf> {
    directories::BaseDirs::new()
        .map(|base| base.home_dir().into())
        .ok_or_else(|| anyhow::anyhow!("Could not get home dir"))
}

pub fn fig_dir() -> Option<PathBuf> {
    Some(directories::BaseDirs::new()?.home_dir().join(".fig"))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_machine_id() {
        let machine_id = get_machine_id();
        assert!(machine_id.is_some());
    }
}
