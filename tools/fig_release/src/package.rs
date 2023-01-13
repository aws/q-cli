use std::path::PathBuf;
use std::time::Duration;

use eyre::bail;
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
    read_channel,
    read_version,
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
    dry: bool,
) -> eyre::Result<()> {
    let version = read_version();
    let channel = read_channel();

    if channel == Channel::None {
        bail!("Can't publish a package with channel set to none");
    }

    if dry {
        bail!("not sure what you expect me to do here")
    }

    let resp = Request::new_release(Method::POST, "/")
        .auth()
        .query(&PublishArgs {
            channel,
            kind,
            architecture,
            version: version.to_string(),
            variant,
        })
        .timeout(Duration::from_secs(120)) // fly uses slow responses to smooth over deploys
        .body(tokio::fs::read(path).await?)
        .send()
        .await?;

    if resp.status().is_success() {
        println!("{}", resp.text().await?);
    } else {
        bail!("error uploading package: {}: {}", resp.status(), resp.text().await?);
    }

    Ok(())
}
