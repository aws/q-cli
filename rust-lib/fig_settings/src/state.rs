use std::path::PathBuf;

use fig_util::directories;

use crate::{
    Error,
    JsonType,
    LocalJson,
};

type Result<T, E = Error> = std::result::Result<T, E>;

pub fn state_path() -> Result<PathBuf> {
    Ok(directories::fig_data_dir()
        .map_err(fig_util::Error::from)?
        .join("state.json"))
}

pub type LocalState = LocalJson;

pub fn local_settings() -> Result<LocalState> {
    LocalState::load(JsonType::State)
}

pub fn get_map() -> Result<serde_json::Map<String, serde_json::Value>> {
    Ok(local_settings()?.inner)
}

pub fn set_value(key: impl Into<String>, value: impl Into<serde_json::Value>) -> Result<()> {
    let mut settings = local_settings()?;
    settings.set(key, value);
    settings.save()?;
    Ok(())
}

pub fn get_value(key: impl AsRef<str>) -> Result<Option<serde_json::Value>> {
    let settings = local_settings()?;
    Ok(settings.get(key).cloned())
}

pub fn get_bool(key: impl AsRef<str>) -> Result<Option<bool>> {
    let settings = local_settings()?;
    let value = settings.get(key);
    Ok(value.cloned().and_then(|v| v.as_bool()))
}

pub fn get_bool_or(key: impl AsRef<str>, default: bool) -> bool {
    get_bool(key).ok().flatten().unwrap_or(default)
}

pub fn get_string(key: impl AsRef<str>) -> Result<Option<String>> {
    let settings = local_settings()?;
    let value = settings.get(key);
    Ok(value.cloned().and_then(|v| v.as_str().map(String::from)))
}

pub fn get_string_or(key: impl AsRef<str>, default: impl Into<String>) -> String {
    get_string(key).ok().flatten().unwrap_or_else(|| default.into())
}

pub fn get_int(key: impl AsRef<str>) -> Result<Option<i64>> {
    let settings = local_settings()?;
    let value = settings.get(key);
    Ok(value.cloned().and_then(|v| v.as_i64()))
}

pub fn get_int_or(key: impl AsRef<str>, default: i64) -> i64 {
    get_int(key).ok().flatten().unwrap_or(default)
}

pub fn remove_value(key: impl AsRef<str>) -> Result<()> {
    let mut settings = local_settings()?;
    settings.remove(key);
    settings.save()?;
    Ok(())
}
