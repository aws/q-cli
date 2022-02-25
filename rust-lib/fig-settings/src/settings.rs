use crate::LocalJson;
use anyhow::{Context, Result};
use directories::BaseDirs;
use std::path::PathBuf;

pub fn settings_path() -> Result<PathBuf> {
    let base_dirs = BaseDirs::new().context("Failed to get base dirs")?;

    let settings_path_1 = base_dirs.config_dir().join("fig").join("settings.json");
    if settings_path_1.exists() {
        return Ok(settings_path_1);
    }

    let settings_path_2 = base_dirs.home_dir().join(".fig").join("settings.json");
    if settings_path_2.exists() {
        return Ok(settings_path_2);
    }

    Err(anyhow::anyhow!("Could not find settings file"))
}

pub type LocalSettings = LocalJson;

pub fn local_settings() -> Result<LocalSettings> {
    let path = settings_path()?;
    LocalSettings::load(path)
}

pub fn set_value(key: impl Into<String>, value: impl Into<serde_json::Value>) -> Result<()> {
    let mut settings = local_settings()?;
    settings.set(key, value)?;
    settings.save()?;
    Ok(())
}

pub fn get_value(key: impl AsRef<str>) -> Result<Option<serde_json::Value>> {
    let settings = local_settings()?;
    Ok(settings.get(key).cloned())
}

pub fn remove_value(key: impl AsRef<str>) -> Result<()> {
    let mut settings = local_settings()?;
    settings.remove(key)?;
    settings.save()?;
    Ok(())
}
