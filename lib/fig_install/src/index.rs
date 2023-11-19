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
use once_cell::sync::Lazy;
use serde::{
    Deserialize,
    Serialize,
};
use strum::EnumString;
use tracing::{
    error,
    info,
    trace,
};

use crate::Error;

static RELEASE_URL: Lazy<&str> = Lazy::new(|| {
    option_env!("CW_BUILD_DESKTOP_RELEASE_URL")
        .unwrap_or("https://desktop-release.codewhisperer.us-east-1.amazonaws.com")
});

#[allow(unused)]
#[derive(Deserialize, Serialize, Debug)]
pub struct Index {
    supported: Vec<Support>,
    versions: Vec<RemoteVersion>,
}

#[allow(unused)]
#[derive(Deserialize, Serialize, Debug)]
struct Support {
    kind: Kind,
    architecture: PackageArchitecture,
    variant: Variant,
}

#[derive(Deserialize, Serialize, Debug)]
struct RemoteVersion {
    version: semver::Version,
    rollout: Option<Rollout>,
    packages: Vec<Package>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Rollout {
    start: u64,
    end: u64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Package {
    kind: Kind,
    architecture: PackageArchitecture,
    variant: Variant,
    download: String,
    sha256: String,
    size: u64,
}

#[derive(Debug)]
pub struct UpdatePackage {
    pub version: String,
    pub download: String,
    pub sha256: String,
    pub size: u64,
}

#[derive(Deserialize, Serialize, PartialEq, Eq, EnumString, Debug)]
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

fn index_endpoint(_channel: &Channel) -> String {
    format!("{}/index.json", *RELEASE_URL)
}

pub async fn pull(channel: &Channel) -> Result<Index, Error> {
    let response = fig_request::client()
        .expect("Unable to create HTTP client")
        .get(index_endpoint(channel))
        .send()
        .await?;
    let index = response.json().await?;
    Ok(index)
}

pub async fn check_for_updates(
    channel: Channel,
    kind: Kind,
    variant: Variant,
    ignore_rollout: bool,
) -> Result<Option<UpdatePackage>, Error> {
    const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
    const ARCHITECTURE: PackageArchitecture = PackageArchitecture::from_system();

    query_index(
        channel,
        kind,
        variant,
        CURRENT_VERSION,
        ARCHITECTURE,
        ignore_rollout,
        None,
    )
    .await
}

pub async fn query_index(
    channel: Channel,
    kind: Kind,
    variant: Variant,
    current_version: &str,
    architecture: PackageArchitecture,
    ignore_rollout: bool,
    threshold_override: Option<u8>,
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

    let Some(sys_id) = get_system_id() else {
        return Err(Error::SystemIdNotFound);
    };
    let system_threshold = threshold_override.unwrap_or_else(|| {
        let mut hasher = DefaultHasher::new();
        // different for each system
        sys_id.hash(&mut hasher);
        // different for each version, which prevents people from getting repeatedly hit by untested
        // releases
        current_version.hash(&mut hasher);

        (hasher.finish() % 0xff) as u8
    });

    let mut chosen = None;
    #[allow(clippy::never_loop)] // todo(mia): fix
    for entry in valid_versions.into_iter() {
        if let Some(rollout) = &entry.rollout {
            if ignore_rollout {
                trace!("accepted update candidate {} because rollout is ignored", entry.version);
                chosen = Some(entry);
                break;
            }
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
                break;
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
                break;
            } else {
                info!(
                    "rejected update candidate {} because remote_threshold {remote_threshold} is below system_threshold {system_threshold}",
                    entry.version
                );
                break;
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
        download: format!("{}/{}", *RELEASE_URL, package.download),
        sha256: package.sha256,
        size: package.size,
    }))
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::*;

    #[test]
    #[ignore]
    fn index_make() {
        let version = |version: &str, sha256: &str, size: u64| RemoteVersion {
            version: Version::parse(version).unwrap(),
            rollout: None,
            packages: vec![Package {
                kind: Kind::Dmg,
                architecture: PackageArchitecture::Universal,
                variant: Variant::Full,
                download: format!("{version}/CodeWhisperer.dmg"),
                sha256: sha256.into(),
                size,
            }],
        };

        let index = Index {
            supported: vec![Support {
                kind: Kind::Dmg,
                architecture: PackageArchitecture::Universal,
                variant: Variant::Full,
            }],
            versions: vec![
                // version(
                //     "0.1.0",
                //     "c588348eb6cc6f4a3b2ececa262ab630e89422d0087fdaf03001499bbb917af0",
                //     93018817,
                // ),
                // version(
                //     "0.2.0",
                //     "1b51608c6d5b8cbc43d05396b1ec74557958df05298f6b6d1efadb203bf9b50a0",
                //     93022923,
                // ),
                // version(
                //     "0.3.0",
                //     "7fff5995557907fb90c4808f5c2ab9307ab94464dcb5703cc9b022d25f1f6718",
                //     93024994,
                // ),
                // version(
                //     "0.4.0",
                //     "21c1145d79cf927a7c6303e40a9933d1efe0dfda52d8bc80e4b9d3ac3643ba7d",
                //     92465710,
                // ),
                // version(
                //     "0.5.0",
                //     "0f85d19c7e90bff7bef16a0643018225465674e0326520171d7e366d47df79d2",
                //     92686534,
                // ),
                // version(
                //     "0.6.0",
                //     "a69a1fec68cd43daa5d80bd6e02c57dfc9e800873a6719d13ad4e20360cb7f9d",
                //     92695962,
                // ),
                version(
                    "0.7.0",
                    "4213d7649e4b1a2ec50adc0266d32d3e1e1f952ed6a863c28d7538190dc92472",
                    82975504,
                ),
                version(
                    "0.8.0",
                    "ee0a8074f094dd2aac3a8d6c136778ab46a1911288d6f2dc9c6f12066578ee4d",
                    82957941,
                ),
                version(
                    "0.9.0",
                    "ec49faa192f3bc01f281676eea16f68053cb1c49f2e18c5fa8addd5946839119",
                    82942146,
                ),
                version(
                    "0.10.0",
                    "3a7c0dd47eb76252c3e3e74b023b35455269dd64255f7083219631d12f6943be",
                    82946880,
                ),
            ],
        };

        let json = serde_json::to_string(&index).unwrap();
        println!("{json}");
        std::fs::write("index.json", json).unwrap();
    }

    #[tokio::test]
    async fn pull_test() {
        let index = pull(&Channel::Stable).await.unwrap();
        println!("{:#?}", index);
        assert!(index.supported.len() > 0);
        assert!(index.versions.len() > 0);
    }

    #[tokio::test]
    async fn check_test() {
        dbg!(
            check_for_updates(Channel::Stable, Kind::Dmg, Variant::Full, false,)
                .await
                .unwrap()
        );
    }
}
