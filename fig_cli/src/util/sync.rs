use std::borrow::Cow;
use std::path::PathBuf;

use eyre::{
    ContextCompat,
    Result,
    WrapErr,
};
use fig_util::directories;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// Sync from a url to a local file
pub trait Sync {
    /// Source to sync from
    fn endpoint(&self) -> Cow<'static, str>;
    /// Destination to sync to
    fn location(&self) -> Result<PathBuf>;
    /// Data to write to the destination
    fn data(&self, _: &[u8]) -> Result<Vec<u8>>;
}

pub struct Settings {}

impl Sync for Settings {
    fn endpoint(&self) -> Cow<'static, str> {
        "/settings".into()
    }

    fn location(&self) -> Result<PathBuf> {
        let fig_dir = directories::fig_dir().context("Could not get fig_dir")?;
        Ok(fig_dir.join("settings.json"))
    }

    fn data(&self, data: &[u8]) -> Result<Vec<u8>> {
        let settings: serde_json::Value = serde_json::from_slice(data).context("Could not parse settings")?;

        let settings = settings.get("settings").context("Could not get settings")?;

        Ok(serde_json::to_vec_pretty(settings)?)
    }
}

pub async fn sync(sync: impl Sync) -> Result<()> {
    let download = fig_request::Request::get(sync.endpoint()).auth().bytes().await?;
    let mut file = File::create(sync.location()?).await?;
    file.write_all(&sync.data(&download)?).await?;
    Ok(())
}
