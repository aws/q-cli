use fig_request::Result;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::{
    json,
    Map,
    Value,
};

pub async fn update_all(settings: Map<String, Value>) -> Result<()> {
    if let Ok(path) = fig_settings::settings::settings_path() {
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
