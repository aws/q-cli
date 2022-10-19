use std::path::PathBuf;
use std::time::Duration;

use fig_request::{
    Method,
    Request,
};
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

    let resp = Request::new_release(Method::POST, "/")
        .auth()
        .query(&PublishArgs {
            channel,
            kind,
            architecture,
            version: release.version,
            variant,
        })
        .timeout(Duration::from_secs(120)) // fly uses slow responses to smooth over deploys
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
