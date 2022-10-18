use std::path::PathBuf;
use std::str::FromStr;

use fig_request::reqwest::Url;
use fig_request::{
    Method,
    Request,
};
use fig_settings::state;
use once_cell::sync::Lazy;
use serde::Serialize;

use crate::cli::{
    PackageArchitecture,
    PackageKind,
    PackageVariant,
};
use crate::utils::{
    read_release_file,
    Channel,
};

static DEPLOY_HOST: Lazy<Url> = Lazy::new(|| {
    Url::from_str(
        &state::get_string("developer.release.apiHost")
            .ok()
            .flatten()
            .unwrap_or_else(|| "https://release.fig.io".into()),
    )
    .unwrap()
});

#[derive(Serialize)]
struct PublishArgs {
    channel: Channel,
    kind: PackageKind,
    architecture: PackageArchitecture,
    version: String,
    variant: PackageVariant,
}

pub async fn package(
    path: PathBuf,
    kind: PackageKind,
    architecture: PackageArchitecture,
    variant: PackageVariant,
) -> eyre::Result<()> {
    let release = read_release_file()?;

    let channel = release
        .channel
        .ok_or_else(|| eyre::eyre!("Can't publish a package without a channel in the release.yaml!"))?;

    let resp = Request::new_with_host(DEPLOY_HOST.clone(), Method::POST, "/")
        .auth()
        .query(&PublishArgs {
            channel,
            kind,
            architecture,
            version: release.version,
            variant,
        })
        .raw_body(tokio::fs::read(path).await?.into())
        .send()
        .await?;

    if resp.status().is_success() {
        println!("{}", resp.text().await?);
    } else {
        println!("error uploading package: {}: {}", resp.status(), resp.text().await?);
        return Ok(());
    }

    Ok(())
}
