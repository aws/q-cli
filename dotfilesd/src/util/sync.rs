use anyhow::{Context, Result};
use fig_auth::get_token;
use std::path::PathBuf;
use tokio::{fs::File, io::AsyncWriteExt};

use reqwest::Url;

use super::fig_dir;

/// Sync from a url to a local file
pub trait Sync {
    /// Source to sync from
    fn source(&self) -> Result<Url>;
    /// Destination to sync to
    fn location(&self) -> Result<PathBuf>;
    /// Data to write to the destination
    fn data(&self, _: &[u8]) -> Result<Vec<u8>>;
}

pub struct Settings {}

impl Sync for Settings {
    fn source(&self) -> Result<Url> {
        Ok(Url::parse("https://api.fig.io/settings")?)
    }

    fn location(&self) -> Result<PathBuf> {
        let fig_dir = fig_dir().context("Could not get fig_dir")?;
        Ok(fig_dir.join("settings.json"))
    }

    fn data(&self, data: &[u8]) -> Result<Vec<u8>> {
        let settings: serde_json::Value =
            serde_json::from_slice(data).context("Could not parse settings")?;

        let settings = settings.get("settings").context("Could not get settings")?;

        Ok(serde_json::to_vec_pretty(settings)?)
    }
}

pub async fn sync(sync: impl Sync) -> Result<()> {
    // Get the token
    let token = get_token().await?;

    let download = reqwest::Client::new()
        .get(sync.source()?)
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    let mut file = File::create(sync.location()?).await?;
    file.write_all(&sync.data(&download)?).await?;

    Ok(())
}
