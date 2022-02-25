use crate::LocalJson;
use anyhow::{Context, Result};
use directories::BaseDirs;
use std::path::PathBuf;

pub fn state_path() -> Result<PathBuf> {
    let base_dirs = BaseDirs::new().context("Failed to get base dirs")?;
    let path = base_dirs.data_dir().join("fig").join("state.json");
    Ok(path)
}

pub type LocalState = LocalJson;

pub fn local_settings() -> Result<LocalState> {
    let path = state_path()?;
    LocalState::load(path)
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
