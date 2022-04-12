use crate::LocalJson;
use std::path::PathBuf;

pub fn state_path() -> Option<PathBuf> {
    fig_directories::fig_data_dir().map(|path| path.join("state.json"))
}

pub type LocalState = LocalJson;

pub fn local_settings() -> Result<LocalState, super::Error> {
    let path = state_path().ok_or(super::Error::SettingsPathNotFound)?;
    LocalState::load(path)
}

pub fn set_value(
    key: impl Into<String>,
    value: impl Into<serde_json::Value>,
) -> Result<(), super::Error> {
    let mut settings = local_settings()?;
    settings.set(key, value)?;
    settings.save()?;
    Ok(())
}

pub fn get_value(key: impl AsRef<str>) -> Result<Option<serde_json::Value>, super::Error> {
    let settings = local_settings()?;
    Ok(settings.get(key).cloned())
}

pub fn get_bool(key: impl AsRef<str>) -> Result<Option<bool>, super::Error> {
    let settings = local_settings()?;
    let value = settings.get(key);
    Ok(value.cloned().and_then(|v| v.as_bool()))
}

pub fn get_string(key: impl AsRef<str>) -> Result<Option<String>, super::Error> {
    let settings = local_settings()?;
    let value = settings.get(key);
    Ok(value.cloned().and_then(|v| v.as_str().map(String::from)))
}

pub fn remove_value(key: impl AsRef<str>) -> Result<(), super::Error> {
    let mut settings = local_settings()?;
    settings.remove(key)?;
    settings.save()?;
    Ok(())
}
