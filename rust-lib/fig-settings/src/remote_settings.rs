use crate::settings::LocalSettings;

use anyhow::Result;
use fig_auth::get_token;

pub type RemoteResult = Result<()>;

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
