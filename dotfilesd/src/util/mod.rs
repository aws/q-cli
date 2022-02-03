use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};
use directories::{BaseDirs, ProjectDirs};
use globset::{Glob, GlobSet, GlobSetBuilder};

pub mod checksum;
pub mod shell;
pub mod terminal;

pub fn project_dir() -> Option<ProjectDirs> {
    directories::ProjectDirs::from("io", "Fig", "Fig Cli")
}

pub fn home_dir() -> Result<PathBuf> {
    directories::BaseDirs::new()
        .map(|base| base.home_dir().to_path_buf())
        .context(anyhow!("Could not get home directory"))
}

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

pub fn fig_dir() -> Option<PathBuf> {
    Some(directories::BaseDirs::new()?.home_dir().join(".fig"))
}

pub fn glob(patterns: &[impl AsRef<str>]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern.as_ref())?);
    }
    Ok(builder.build()?)
}

pub struct Settings {
    inner: serde_json::Value,
}

impl Settings {
    pub fn load() -> Result<Self> {
        let settings_path = BaseDirs::new()
            .context("Could not get home dir")?
            .home_dir()
            .join(".fig")
            .join("settings.json");

        let settings_file = fs::read_to_string(settings_path)?;

        Ok(Self {
            inner: serde_json::from_str(&settings_file)?,
        })
    }

    pub fn save(&self) -> Result<()> {
        let settings_path = BaseDirs::new()
            .context("Could not get home dir")?
            .home_dir()
            .join(".fig")
            .join("settings.json");

        fs::write(settings_path, serde_json::to_string_pretty(&self.inner)?)?;
        Ok(())
    }

    pub fn get_mut_settings(&mut self) -> Option<&mut serde_json::Map<String, serde_json::Value>> {
        self.inner.as_object_mut()
    }

    pub fn get_setting(&self) -> Option<&serde_json::Map<String, serde_json::Value>> {
        self.inner.as_object()
    }
}

#[cfg(target_os = "macos")]
pub fn get_machine_id() -> Option<String> {
    use std::process::Command;

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
        .to_string();

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
