use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{
    Hash,
    Hasher,
};
use std::time::{
    SystemTime,
    UNIX_EPOCH,
};

use fig_util::system_info::{
    get_arch,
    get_platform,
    get_system_id,
};
use once_cell::sync::Lazy;
use reqwest::Client;
use semver::Version;
use serde::Deserialize;
use tracing::trace;

use crate::Error;

#[derive(Deserialize)]
pub struct Index {
    latest_version: String,
    versions: HashMap<String, VersionInfo>,
}

#[derive(Deserialize)]
pub struct VersionInfo {
    windows: Windows,
    macos: Macos,
    rollout: Option<Rollout>,
}

#[derive(Deserialize)]
pub struct Rollout {
    start: u64,
    end: u64,
}

#[derive(Deserialize, Clone, Debug)]
/// Resolved update package
pub struct Package {
    /// A link that can be used to download the package
    pub download: String,
    /// SHA256 of the entire downloaded object
    // todo(mia): automatically verify this instead of delegating to platform-specific code
    pub sha256: String,
}

#[derive(Deserialize)]
pub struct Windows {
    x86_64: Package,
}

#[derive(Deserialize)]
pub struct Macos {
    x86_64: Package,
    aarch64: Package,
}

static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .user_agent(concat!("fig_update/", env!("CARGO_PKG_VERSION")))
        .build()
        .expect("Failed building update client")
});

const INDEX_ENDPOINT: &str = "https://pkg.fig.io/managed/index";

async fn pull() -> Result<Index, Error> {
    let response = CLIENT.get(INDEX_ENDPOINT).send().await?;
    let index = response.json().await?;
    Ok(index)
}

pub async fn check(current_version: &str) -> Result<Option<Package>, Error> {
    let index = pull().await?;

    let remote_version = Version::parse(&index.latest_version)?;
    let local_version = Version::parse(current_version)?;

    if remote_version <= local_version {
        return Ok(None); // remote version isn't higher than current version
    }

    let mut remote_versions = index
        .versions
        .keys()
        .filter_map(|key| {
            Version::parse(key)
                .ok()
                .and_then(|x| if x > local_version { Some((x, key)) } else { None })
        })
        .collect::<Vec<_>>();

    remote_versions.sort_by_cached_key(|x| x.0.clone());
    remote_versions.reverse();

    let mut chosen = None;
    let right_now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    let system_threshold = {
        let mut hasher = DefaultHasher::new();
        // different for each system
        get_system_id()?.hash(&mut hasher);
        // different for each version, which prevents people from getting repeatedly hit by untested
        // releases
        current_version.hash(&mut hasher);

        (hasher.finish() % 0xff) as u8
    };

    for remote_version in remote_versions.into_iter().map(|x| x.1) {
        if let Some(entry) = index.versions.get(remote_version) {
            if let Some(rollout) = &entry.rollout {
                if rollout.end < right_now {
                    trace!("accepted update candidate {remote_version} because rollout is over");
                    chosen = Some(entry);
                    break;
                }
                if rollout.start > right_now {
                    trace!("rejected update candidate {remote_version} because rollout hasn't started yet");
                    continue;
                }

                // interpolate rollout progress
                let offset_into = (right_now - rollout.start) as f64;
                let rollout_length = (rollout.end - rollout.start) as f64;
                let progress = offset_into / rollout_length;
                let remote_threshold = (progress * 256.0).round().clamp(0.0, 256.0) as u8;

                if remote_threshold >= system_threshold {
                    // the rollout chose us
                    chosen = Some(entry);
                    trace!(
                        "accepted update candidate {remote_version} with remote_threshold {remote_threshold} and system_threshold {system_threshold}"
                    );
                } else {
                    trace!(
                        "rejected update candidate {remote_version} because remote_threshold {remote_threshold} is below system_threshold {system_threshold}"
                    );
                }
            } else {
                chosen = Some(entry);
                break;
            }
        }
    }

    if chosen.is_none() {
        // no upgrade candidates
        return Ok(None);
    }

    let candidate = chosen.unwrap();

    let package = match (get_platform(), get_arch()) {
        ("windows", "x86_64") => candidate.windows.x86_64.clone(),
        ("macos", "x86_64") => candidate.macos.x86_64.clone(),
        ("macos", "aarch64") => candidate.macos.aarch64.clone(),
        _ => return Err(Error::UnsupportedPlatform),
    };

    Ok(Some(package))
}
