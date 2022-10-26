use std::io::Write;

use fig_request::Result;
use fig_util::directories;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::{
    json,
    Map,
    Value,
};
use time::format_description::well_known::Rfc3339;

pub async fn update_all(settings: Map<String, Value>) -> Result<()> {
    if let Ok(path) = directories::settings_path() {
        if let Ok(file) = std::fs::File::open(path) {
            serde_json::to_writer_pretty(file, &settings).ok();
        }
    }
    fig_request::Request::post("/settings/update")
        .body(&json!({ "settings": settings }))
        .auth()
        .send()
        .await?;
    Ok(())
}

async fn update_remote(key: impl AsRef<str>, value: impl Into<serde_json::Value>) -> Result<()> {
    let value = value.into();
    fig_request::Request::post(format!("/settings/update/{}", key.as_ref()))
        .body(&json!({ "value": value }))
        .auth()
        .send()
        .await?;
    Ok(())
}

pub async fn update(key: impl AsRef<str>, value: impl Into<serde_json::Value>) -> Result<()> {
    let value = value.into();
    fig_settings::settings::set_value(key.as_ref(), value.clone()).ok();
    update_remote(key, value).await?;
    Ok(())
}

pub async fn delete(key: impl AsRef<str>) -> Result<()> {
    fig_settings::settings::remove_value(key.as_ref()).ok();
    fig_request::Request::post(format!("/settings/update/{}", key.as_ref()))
        .auth()
        .send()
        .await?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub settings: serde_json::Map<String, Value>,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: time::OffsetDateTime,
}

/// Ensure that telemetry setting from pre-login is respected
///
/// Currently we sync the settings on login, but we need to ensure that the telemetry setting is
/// respected from before login. This will do a one-time migration of the setting on login.
pub async fn ensure_telemetry(settings: &mut Map<String, Value>) -> Result<()> {
    // If we have never set the telemetry from this fn, it is set locally, and not in the settings
    // passed in, we need to set it in that map and send it to the server
    if !fig_settings::state::get_bool_or("telemetry.setOnRemote", false)
        && fig_settings::settings::get_bool_or("telemetry.disabled", false)
        && !settings.contains_key("telemetry.disabled")
    {
        fig_settings::state::set_value("telemetry.setOnRemote", true).ok();
        settings.insert("telemetry.disabled".to_string(), json!(true));
        update_remote("telemetry.disabled", true).await?;
    }

    Ok(())
}

pub async fn get() -> Result<Settings> {
    fig_request::Request::get("/settings").auth().deser_json().await
}

pub async fn sync() -> Result<()> {
    let Settings {
        mut settings,
        updated_at,
    } = get().await?;

    ensure_telemetry(&mut settings).await?;

    let path = directories::settings_path()?;

    let mut settings_file = std::fs::File::create(&path)?;
    let settings_json = serde_json::to_string_pretty(&settings)?;
    settings_file.write_all(settings_json.as_bytes())?;

    if let Ok(updated_at) = updated_at.format(&Rfc3339) {
        fig_settings::state::set_value("settings.updatedAt", json!(updated_at)).ok();
    }

    Ok(())
}
