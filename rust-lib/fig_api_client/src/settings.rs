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

pub async fn update(key: impl AsRef<str>, value: impl Into<serde_json::Value>) -> Result<()> {
    let value = value.into();
    fig_settings::settings::set_value(key.as_ref(), value.clone()).ok();
    fig_request::Request::post(format!("/settings/update/{}", key.as_ref()))
        .body(&json!({ "value": value }))
        .auth()
        .send()
        .await?;
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
    pub settings: serde_json::Value,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: time::OffsetDateTime,
}

pub async fn get() -> Result<Settings> {
    fig_request::Request::get("/settings").auth().deser_json().await
}

pub async fn sync() -> Result<()> {
    let Settings { settings, updated_at } = get().await?;

    let path = directories::settings_path()?;

    let mut settings_file = std::fs::File::create(&path)?;
    let settings_json = serde_json::to_string_pretty(&settings)?;
    settings_file.write_all(settings_json.as_bytes())?;

    if let Ok(updated_at) = updated_at.format(&Rfc3339) {
        fig_settings::state::set_value("settings.updatedAt", json!(updated_at)).ok();
    }

    Ok(())
}
