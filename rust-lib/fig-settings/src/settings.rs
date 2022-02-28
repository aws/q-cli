use crate::LocalJson;
use anyhow::{Context, Result};
use directories::BaseDirs;
use fig_auth::get_token;
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
pub type RemoteResult = Result<()>;

pub fn local_settings() -> Result<LocalSettings> {
    let path = settings_path()?;
    LocalSettings::load(path)
}

pub async fn update_remote(settings: LocalSettings) -> RemoteResult {
    if let Some(settings) = settings.get_setting() {
        let token = get_token().await?;
        let mut body = serde_json::Map::new();
        body.insert("settings".into(), serde_json::json!(settings));

        reqwest::Client::new()
            .post("https://api.fig.io/settings/update")
            .header("Content-Type", "application/json")
            .json(&body)
            .bearer_auth(token)
            .send()
            .await?
            .error_for_status()?;
    }

    Ok(())
}

pub async fn set_value(
    key: impl Into<String>,
    value: impl Into<serde_json::Value>,
) -> Result<RemoteResult> {
    let mut settings = local_settings()?;
    settings.set(key, value)?;
    settings.save()?;
    Ok(update_remote(settings).await)
}

pub fn get_value(key: impl AsRef<str>) -> Result<Option<serde_json::Value>> {
    let settings = local_settings()?;
    let value = settings.get(key);
    Ok(value.cloned())
}

pub async fn remove_value(key: impl AsRef<str>) -> Result<RemoteResult> {
    let mut settings = local_settings()?;
    settings.remove(key)?;
    settings.save()?;
    Ok(update_remote(settings).await)
}
