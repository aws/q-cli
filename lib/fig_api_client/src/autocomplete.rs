use std::borrow::Cow;

use fig_request::{
    reqwest_client,
    Error,
};

use crate::clients::github::{
    GitHub,
    GithubRelease,
};

pub const AUTOCOMPLETE_REPO: GitHub = GitHub::new(Cow::Borrowed("withfig"), Cow::Borrowed("autocomplete"));

pub async fn get_zipped_specs_from(release: &GithubRelease) -> Result<Vec<u8>, Error> {
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
pub async fn get_zipped_specs() -> Result<Vec<u8>, Error> {
    let release = AUTOCOMPLETE_REPO.latest_release().await?;
    get_zipped_specs_from(&release).await
}
