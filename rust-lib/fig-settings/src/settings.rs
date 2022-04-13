use crate::{
    remote_settings::{delete_remote_setting, update_remote_setting},
    Error, LocalJson,
};
use std::path::PathBuf;

pub fn settings_path() -> Option<PathBuf> {
    fig_directories::fig_dir().map(|path| path.join("settings.json"))
}

pub type LocalSettings = LocalJson;

pub fn local_settings() -> Result<LocalSettings, super::Error> {
    let path = settings_path().ok_or(super::Error::SettingsPathNotFound)?;
    LocalSettings::load(path)
}

pub async fn set_value(
    key: impl Into<String>,
    value: impl Into<serde_json::Value>,
) -> Result<(), super::Error> {
    let key = key.into();
    let value = value.into();
    let mut settings = local_settings()?;
    settings.set(&key, value.clone())?;
    settings.save()?;
    Ok(update_remote_setting(key, value).await?)
}

pub fn get_value(key: impl AsRef<str>) -> Result<Option<serde_json::Value>, super::Error> {
    let settings = local_settings()?;
    let value = settings.get(key);
    Ok(value.cloned())
}

pub fn get_bool(key: impl AsRef<str>) -> Result<Option<bool>, Error> {
    let settings = local_settings()?;
    let value = settings.get(key);
    Ok(value.cloned().and_then(|v| v.as_bool()))
}

pub fn get_bool_or(key: impl AsRef<str>, default: bool) -> bool {
    get_bool(key).ok().flatten().unwrap_or(default)
}

pub fn get_string(key: impl AsRef<str>) -> Result<Option<String>, Error> {
    let settings = local_settings()?;
    let value = settings.get(key);
    Ok(value.cloned().and_then(|v| v.as_str().map(String::from)))
}

pub fn get_string_or(key: impl AsRef<str>, default: String) -> String {
    get_string(key).ok().flatten().unwrap_or(default)
}

pub async fn remove_value(key: impl AsRef<str>) -> Result<(), Error> {
    let mut settings = local_settings()?;
    settings.remove(&key)?;
    settings.save()?;
    Ok(delete_remote_setting(&key).await?)
}
