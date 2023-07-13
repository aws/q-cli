use fig_request::{
    reqwest_client,
    Error,
    Result,
};
use once_cell::sync::Lazy;

use crate::clients::github::{
    GitHub,
    GithubRelease,
};

pub static AUTOCOMPLETE_REPO: Lazy<GitHub> = Lazy::new(|| GitHub::new("withfig", "autocomplete"));

pub async fn get_zipped_specs_from(release: &GithubRelease) -> Result<Vec<u8>> {
    let asset = release.assets.first().unwrap();
    let Some(client) = reqwest_client::reqwest_client(true) else {
        return Err(Error::NoClient);
    };
    Ok(client
        .get(asset.browser_download_url.clone())
        .header("Accept", &asset.content_type)
        .send()
        .await?
        .bytes()
        .await?
        .into())
}

#[inline(always)]
pub async fn get_zipped_specs() -> Result<Vec<u8>> {
    let release = AUTOCOMPLETE_REPO.latest_release().await?;
    get_zipped_specs_from(&release).await
}
