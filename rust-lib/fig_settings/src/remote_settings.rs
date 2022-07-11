use std::fmt::Display;

use fig_auth::get_token;
use serde::{
    Deserialize,
    Serialize,
};
use thiserror::Error;

use crate::api_host;
use crate::settings::LocalSettings;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    AuthError(#[from] fig_auth::Error),
}

pub async fn update_remote_all_settings(settings: LocalSettings) -> Result<(), Error> {
    let token = get_token().await?;

    let mut body = serde_json::Map::new();
    body.insert("settings".into(), serde_json::json!(&settings.inner));

    let mut url = api_host();
    url.set_path("/settings/update");

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

pub async fn update_remote_setting(key: impl Display, value: impl Into<serde_json::Value>) -> Result<(), Error> {
    let token = get_token().await?;

    let mut body = serde_json::Map::new();
    body.insert("value".into(), value.into());

    let mut url = api_host();
    url.set_path(&format!("/settings/update/{key}"));

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

pub async fn delete_remote_setting(key: impl Display) -> Result<(), Error> {
    let token = get_token().await?;

    let mut url = api_host();
    url.set_path(&format!("/settings/update/{key}"));

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

pub async fn get_settings() -> Result<RemoteSettings, Error> {
    let token = get_token().await?;

    let mut url = api_host();
    url.set_path("/settings");

    let res = reqwest::Client::new()
        .get(url)
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?;

    let body: RemoteSettings = res.json().await?;

    Ok(body)
}
