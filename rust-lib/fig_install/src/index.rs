use std::collections::hash_map::DefaultHasher;
use std::hash::{
    Hash,
    Hasher,
};
use std::time::{
    SystemTime,
    UNIX_EPOCH,
};

use cfg_if::cfg_if;
use fig_util::manifest::{
    Channel,
    Kind,
    Variant,
};
use fig_util::system_info::get_system_id;
use semver::Version;
use serde::Deserialize;
use strum::EnumString;
use tracing::{
    error,
    info,
    trace,
};

use crate::Error;

#[allow(unused)]
#[derive(Deserialize)]
struct Index {
    supported: Vec<Support>,
    versions: Vec<RemoteVersion>,
}

#[allow(unused)]
#[derive(Deserialize)]
struct Support {
    kind: Kind,
    architecture: PackageArchitecture,
    variant: Variant,
}

#[derive(Deserialize, Debug)]
struct RemoteVersion {
    version: semver::Version,
    rollout: Option<Rollout>,
    packages: Vec<Package>,
}

#[derive(Deserialize, Debug)]
struct Rollout {
    start: u64,
    end: u64,
}

#[derive(Deserialize, Debug)]
pub struct Package {
    kind: Kind,
    architecture: PackageArchitecture,
    variant: Variant,
    download: String,
    sha256: String,
}

#[derive(Debug)]
pub struct UpdatePackage {
    pub version: String,
    pub download: String,
    pub sha256: String,
}

#[derive(Deserialize, PartialEq, Eq, EnumString, Debug)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum PackageArchitecture {
    #[serde(rename = "x86_64")]
    #[strum(serialize = "x86_64")]
    X86_64,
    #[serde(rename = "aarch64")]
    #[strum(serialize = "aarch64")]
    AArch64,
    Universal,
}

impl PackageArchitecture {
    const fn from_system() -> Self {
        cfg_if! {
            if #[cfg(target_os = "macos")] {
                PackageArchitecture::Universal
            } else if #[cfg(target_arch = "x86_64")] {
                PackageArchitecture::X86_64
            } else if #[cfg(target_arch = "aarch64")] {
                PackageArchitecture::AArch64
            } else {
                compile_error!("unknown architecture")
            }
        }
    }
}

fn index_endpoint(channel: &Channel) -> &'static str {
    match channel {
        Channel::Nightly => "https://repo.fig.io/generic/nightly/index.json",
        Channel::Qa => "https://repo.fig.io/generic/qa/index.json",
        Channel::Beta => "https://repo.fig.io/generic/beta/index.json",
        Channel::Stable => "https://repo.fig.io/generic/stable/index.json",
    }
}

#[deprecated = "versions are unified, use env!(\"CARGO_PKG_VERSION\")"]
pub fn local_manifest_version() -> Result<Version, Error> {
    Ok(Version::parse(env!("CARGO_PKG_VERSION"))?)
}

async fn pull(channel: &Channel) -> Result<Index, Error> {
    let response = fig_request::client()
        .expect("Unable to create HTTP client")
        .get(index_endpoint(channel))
        .send()
        .await?;
    let index = response.json().await?;
    Ok(index)
}

pub async fn check_for_updates(channel: Channel, kind: Kind, variant: Variant) -> Result<Option<UpdatePackage>, Error> {
    const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
    const ARCHITECTURE: PackageArchitecture = PackageArchitecture::from_system();

    query_index(channel, kind, variant, CURRENT_VERSION, ARCHITECTURE).await
}

pub async fn query_index(
    channel: Channel,
    kind: Kind,
    variant: Variant,
    current_version: &str,
    architecture: PackageArchitecture,
) -> Result<Option<UpdatePackage>, Error> {
    let index = pull(&channel).await?;

    if !index
        .supported
        .iter()
        .any(|support| support.kind == kind && support.architecture == architecture && support.variant == variant)
    {
        return Err(Error::SystemNotOnChannel);
    }

    let right_now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    let mut valid_versions = index
        .versions
        .into_iter()
        .filter(|version| {
            version.packages.iter().any(|package| {
                package.kind == kind && package.architecture == architecture && package.variant == variant
            })
        })
        .filter(|version| match &version.rollout {
            Some(rollout) => rollout.start <= right_now,
            None => true,
        })
        .collect::<Vec<RemoteVersion>>();

    valid_versions.sort_unstable_by(|lhs, rhs| lhs.version.cmp(&rhs.version));
    valid_versions.reverse();

    let system_threshold = {
        let mut hasher = DefaultHasher::new();
        // different for each system
        get_system_id()?.hash(&mut hasher);
        // different for each version, which prevents people from getting repeatedly hit by untested
        // releases
        current_version.hash(&mut hasher);

        (hasher.finish() % 0xff) as u8
    };

    let mut chosen = None;
    for entry in valid_versions.into_iter() {
        if let Some(rollout) = &entry.rollout {
            if rollout.end < right_now {
                trace!("accepted update candidate {} because rollout is over", entry.version);
                chosen = Some(entry);
                break;
            }
            if rollout.start > right_now {
                trace!(
                    "rejected update candidate {} because rollout hasn't started yet",
                    entry.version
                );
                continue;
            }

            // interpolate rollout progress
            let offset_into = (right_now - rollout.start) as f64;
            let rollout_length = (rollout.end - rollout.start) as f64;
            let progress = offset_into / rollout_length;
            let remote_threshold = (progress * 256.0).round().clamp(0.0, 256.0) as u8;

            if remote_threshold >= system_threshold {
                // the rollout chose us
                info!(
                    "accepted update candidate {} with remote_threshold {remote_threshold} and system_threshold {system_threshold}",
                    entry.version
                );
                chosen = Some(entry);
            } else {
                info!(
                    "rejected update candidate {} because remote_threshold {remote_threshold} is below system_threshold {system_threshold}",
                    entry.version
                );
            }
        } else {
            chosen = Some(entry);
            break;
        }
    }

    if chosen.is_none() {
        // no upgrade candidates
        return Ok(None);
    }

    let chosen = chosen.unwrap();
    let package = chosen
        .packages
        .into_iter()
        .find(|package| package.kind == kind && package.architecture == architecture && package.variant == variant)
        .unwrap();

    if match semver::Version::parse(current_version) {
        Ok(current_version) => chosen.version <= current_version,
        Err(err) => {
            error!("failed parsing current version semver: {err:?}");
            chosen.version.to_string() == current_version
        },
    } {
        return Ok(None);
    }

    Ok(Some(UpdatePackage {
        version: chosen.version.to_string(),
        download: package.download,
        sha256: package.sha256,
    }))
}
