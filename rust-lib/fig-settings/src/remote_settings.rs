use crate::{api_host, settings::LocalSettings};

use anyhow::Result;
use fig_auth::get_token;
use reqwest::Url;
use serde::{Deserialize, Serialize};

pub type RemoteResult = Result<()>;

pub async fn update_remote_all_settings(settings: LocalSettings) -> RemoteResult {
    if let Some(settings) = settings.get_setting() {
        let token = get_token().await?;
        let mut body = serde_json::Map::new();
        body.insert("settings".into(), serde_json::json!(settings));

        let api_host = api_host();
        let url = Url::parse(&format!("{api_host}/settings/update"))?;

        reqwest::Client::new()
            .post(url)
            .header("Content-Type", "application/json")
            .json(&body)
            .bearer_auth(token)
            .send()
            .await?
            .error_for_status()?;
    }

    Ok(())
}

pub async fn update_remote_setting(
    key: impl AsRef<str>,
    value: impl Into<serde_json::Value>,
) -> RemoteResult {
    let token = get_token().await?;

    let mut body = serde_json::Map::new();
    body.insert("value".into(), value.into());

    let api_host = api_host();
    let url = Url::parse(&format!("{api_host}/settings/update/{}", key.as_ref()))?;

    reqwest::Client::new()
        .post(url)
        .header("Content-Type", "application/json")
        .json(&body)
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

pub async fn delete_remote_setting(key: impl AsRef<str>) -> RemoteResult {
    let token = get_token().await?;

    let api_host = api_host();
    let url = Url::parse(&format!("{api_host}/settings/update/{}", key.as_ref()))?;

    reqwest::Client::new()
        .delete(url)
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSettings {
    pub settings: serde_json::Value,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: time::OffsetDateTime,
}

pub async fn get_settings() -> Result<RemoteSettings> {
    let token = get_token().await?;

    let api_host = api_host();
    let url = Url::parse(&format!("{api_host}/settings/"))?;

    let res = reqwest::Client::new()
        .get(url)
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?;

    let body: RemoteSettings = res.json().await?;

    Ok(body)
}
