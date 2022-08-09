use std::path::PathBuf;

use fig_util::directories;

use crate::{
    Error,
    LocalJson,
};

type Result<T, E = Error> = std::result::Result<T, E>;

pub fn settings_path() -> Result<PathBuf> {
    Ok(directories::fig_dir()
        .map_err(fig_util::Error::from)?
        .join("settings.json"))
}

pub type LocalSettings = LocalJson;

pub fn local_settings() -> Result<LocalSettings> {
    let path = settings_path()?;
    LocalSettings::load(path)
}

pub fn get_map() -> Result<serde_json::Map<String, serde_json::Value>> {
    Ok(local_settings()?.inner)
}

/// Do not use this if you want to update remote settings, use
/// [fig_api_client::settings::update]
pub fn set_value(key: impl Into<String>, value: impl Into<serde_json::Value>) -> Result<()> {
    let key = key.into();
    let value = value.into();
    let mut settings = local_settings()?;
    settings.set(&key, value);
    settings.save()?;
    Ok(())
}

/// Do not use this if you want to update remote settings_path, use
/// [fig_api_client::settings::delete]
pub fn remove_value(key: impl AsRef<str>) -> Result<()> {
    let mut settings = local_settings()?;
    settings.remove(&key);
    settings.save()?;
    Ok(())
}

pub fn get_value(key: impl AsRef<str>) -> Result<Option<serde_json::Value>> {
    let settings = local_settings()?;
    let value = settings.get(key);
    Ok(value.cloned())
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

pub fn get_string_or(key: impl AsRef<str>, default: String) -> String {
    get_string(key).ok().flatten().unwrap_or(default)
}

pub fn get_int(key: impl AsRef<str>) -> Result<Option<i64>> {
    let settings = local_settings()?;
    let value = settings.get(key);
    Ok(value.cloned().and_then(|v| v.as_i64()))
}

pub fn get_int_or(key: impl AsRef<str>, default: i64) -> i64 {
    get_int(key).ok().flatten().unwrap_or(default)
}

pub fn product_gate(product: impl std::fmt::Display, namespace: Option<impl std::fmt::Display>) -> Result<bool> {
    let settings = local_settings()?;
    match namespace {
        Some(namespace) => Ok(settings
            .get(&format!("product-gate.{namespace}.{product}.enabled"))
            .and_then(|val| val.as_bool())
            .unwrap_or_default()),
        None => Ok(settings
            .get(&format!("product-gate.{product}.enabled"))
            .and_then(|val| val.as_bool())
            .unwrap_or_default()
            || settings
                .get(&format!("{product}.beta"))
                .and_then(|val| val.as_bool())
                .unwrap_or_default()),
    }
}
