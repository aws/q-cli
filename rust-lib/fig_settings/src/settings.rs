use std::fmt::Display;
use std::path::PathBuf;

use fig_util::directories;

use crate::remote_settings::{
    delete_remote_setting,
    update_remote_setting,
};
use crate::{
    Error,
    LocalJson,
};

pub fn settings_path() -> Result<PathBuf, super::Error> {
    Ok(directories::fig_dir()
        .map_err(fig_util::Error::from)?
        .join("settings.json"))
}

pub type LocalSettings = LocalJson;

pub fn local_settings() -> Result<LocalSettings, super::Error> {
    let path = settings_path()?;
    LocalSettings::load(path)
}

pub fn get_map() -> Result<serde_json::Map<String, serde_json::Value>, super::Error> {
    Ok(local_settings()?.inner)
}

pub async fn set_value(key: impl Into<String>, value: impl Into<serde_json::Value>) -> Result<(), super::Error> {
    let key = key.into();
    let value = value.into();
    let mut settings = local_settings()?;
    settings.set(&key, value.clone());
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

pub fn get_int(key: impl AsRef<str>) -> Result<Option<i64>, super::Error> {
    let settings = local_settings()?;
    let value = settings.get(key);
    Ok(value.cloned().and_then(|v| v.as_i64()))
}

pub fn get_int_or(key: impl AsRef<str>, default: i64) -> i64 {
    get_int(key).ok().flatten().unwrap_or(default)
}

pub async fn remove_value(key: impl AsRef<str>) -> Result<(), Error> {
    let mut settings = local_settings()?;
    settings.remove(&key);
    settings.save()?;
    Ok(delete_remote_setting(key.as_ref()).await?)
}

pub async fn product_gate(product: impl Display, namespace: Option<impl Display>) -> bool {
    let settings = match local_settings() {
        Ok(settings) => settings,
        Err(_) => return false,
    };
    settings
        .get(&format!("product-gate.{product}.enabled"))
        .and_then(|val| val.as_bool())
        .unwrap_or_default()
        || if let Some(namespace) = namespace {
            settings
                .get(&format!("product-gate.{namespace}.{product}.enabled"))
                .and_then(|val| val.as_bool())
                .unwrap_or_default()
        } else {
            false
        }
        || settings
            .get(&format!("{product}.beta"))
            .and_then(|val| val.as_bool())
            .unwrap_or_default()
}
