use crate::settings::LocalSettings;

use anyhow::Result;
use fig_auth::get_token;

pub type RemoteResult = Result<()>;

pub async fn update_remote_all_settings(settings: LocalSettings) -> RemoteResult {
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

pub async fn update_remote_setting(
    key: impl AsRef<str>,
    value: impl Into<serde_json::Value>,
) -> RemoteResult {
    let token = get_token().await?;

    let mut body = serde_json::Map::new();
    body.insert("value".into(), value.into());

    let url = reqwest::Url::parse(&format!(
        "https://api.fig.io/settings/update/{}",
        key.as_ref()
    ))?;

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

    let url = reqwest::Url::parse(&format!(
        "https://api.fig.io/settings/update/{}",
        key.as_ref()
    ))?;

    reqwest::Client::new()
        .delete(url)
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}
