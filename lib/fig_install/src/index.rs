use std::collections::hash_map::DefaultHasher;
use std::hash::{
    Hash,
    Hasher,
};
use std::sync::OnceLock;
use std::time::{
    SystemTime,
    UNIX_EPOCH,
};

use cfg_if::cfg_if;
use fig_util::manifest::{
    Channel,
    FileType,
    Os,
    Variant,
};
use fig_util::system_info::get_system_id;
use semver::Version;
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
use url::Url;

use crate::Error;

const DEFAULT_RELEASE_URL: &str = "https://desktop-release.codewhisperer.us-east-1.amazonaws.com";

/// The url to check for updates from, tries the following order:
/// - The env var `Q_DESKTOP_RELEASE_URL`
/// - The setting `install.releaseUrl`
/// - Falls back to the default or the build time env var `Q_BUILD_DESKTOP_RELEASE_URL`
fn release_url() -> &'static Url {
    static RELEASE_URL: OnceLock<Url> = OnceLock::new();
    RELEASE_URL.get_or_init(|| {
        match std::env::var("Q_DESKTOP_RELEASE_URL") {
            Ok(s) => Url::parse(&s),
            Err(_) => match fig_settings::settings::get_string("install.releaseUrl") {
                Ok(Some(s)) => Url::parse(&s),
                _ => Url::parse(option_env!("Q_BUILD_DESKTOP_RELEASE_URL").unwrap_or(DEFAULT_RELEASE_URL)),
            },
        }
        .unwrap()
    })
}

fn deser_enum_other<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    match T::from_str(<&str as Deserialize<'de>>::deserialize(deserializer)?) {
        Ok(s) => Ok(s),
        Err(err) => Err(serde::de::Error::custom(err)),
    }
}

fn deser_opt_enum_other<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    match Option::<&'de str>::deserialize(deserializer)? {
        Some(s) => match T::from_str(s) {
            Ok(s) => Ok(Some(s)),
            Err(err) => Err(serde::de::Error::custom(err)),
        },
        None => Ok(None),
    }
}

#[allow(unused)]
#[derive(Deserialize, Serialize, Debug)]
pub struct Index {
    supported: Vec<Support>,
    versions: Vec<RemoteVersion>,
}

impl Index {
    #[allow(dead_code)]
    pub(crate) fn latest(&self) -> Option<&RemoteVersion> {
        self.versions.iter().max_by(|a, b| a.version.cmp(&b.version))
    }
}

#[allow(unused)]
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Support {
    #[serde(deserialize_with = "deser_enum_other")]
    architecture: PackageArchitecture,
    #[serde(deserialize_with = "deser_enum_other")]
    variant: Variant,
    #[serde(deserialize_with = "deser_opt_enum_other", default)]
    os: Option<Os>,
    #[serde(deserialize_with = "deser_opt_enum_other", default)]
    file_type: Option<FileType>,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct RemoteVersion {
    pub version: Version,
    pub rollout: Option<Rollout>,
    pub packages: Vec<Package>,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct Rollout {
    start: u64,
    end: u64,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    #[serde(deserialize_with = "deser_enum_other")]
    pub(crate) architecture: PackageArchitecture,
    #[serde(deserialize_with = "deser_enum_other")]
    pub(crate) variant: Variant,
    #[serde(deserialize_with = "deser_opt_enum_other", default)]
    pub(crate) os: Option<Os>,
    #[serde(deserialize_with = "deser_opt_enum_other", default)]
    pub(crate) file_type: Option<FileType>,
    pub(crate) download: String,
    pub(crate) sha256: String,
    pub(crate) size: u64,
    pub(crate) cli_path: Option<String>,
}

impl Package {
    pub(crate) fn download_url(&self) -> Url {
        let mut url = release_url().clone();
        url.set_path(&self.download);
        url
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct UpdatePackage {
    /// The version of the package
    pub version: Version,
    /// The url to download the archive from
    pub download_url: Url,
    /// The sha256 sum of the archive
    pub sha256: String,
    /// Size of the package in bytes
    pub size: u64,
    /// Path to the CLI in the bundle
    pub cli_path: Option<String>,
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
    #[strum(default)]
    Other(String),
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

fn index_endpoint(_channel: &Channel) -> Url {
    let mut url = release_url().clone();
    url.set_path("index.json");
    url
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
    os: Os,
    variant: Variant,
    ignore_rollout: bool,
) -> Result<Option<UpdatePackage>, Error> {
    const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
    const ARCHITECTURE: PackageArchitecture = PackageArchitecture::from_system();
    const FILE_TYPE: FileType = FileType::from_system();

    query_index(
        channel,
        os,
        variant,
        FILE_TYPE,
        CURRENT_VERSION,
        ARCHITECTURE,
        ignore_rollout,
        None,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn query_index(
    channel: Channel,
    os: Os,
    variant: Variant,
    file_type: FileType,
    current_version: &str,
    architecture: PackageArchitecture,
    ignore_rollout: bool,
    threshold_override: Option<u8>,
) -> Result<Option<UpdatePackage>, Error> {
    let index = pull(&channel).await?;

    if !index.supported.iter().any(|support| {
        support.os.as_ref() == Some(&os)
            && support.architecture == architecture
            && support.variant == variant
            && support.file_type.as_ref() == Some(&file_type)
    }) {
        return Err(Error::SystemNotOnChannel);
    }

    let right_now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    let mut valid_versions = index
        .versions
        .into_iter()
        .filter(|version| {
            version.packages.iter().any(|package| {
                package.os.as_ref() == Some(&os)
                    && package.architecture == architecture
                    && package.variant == variant
                    && package.file_type.as_ref() == Some(&file_type)
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
    for entry in valid_versions {
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
        .find(|package| {
            package.os.as_ref() == Some(&os)
                && package.architecture == architecture
                && package.variant == variant
                && package.file_type.as_ref() == Some(&file_type)
        })
        .unwrap();

    if match Version::parse(current_version) {
        Ok(current_version) => chosen.version <= current_version,
        Err(err) => {
            error!("failed parsing current version semver: {err:?}");
            chosen.version.to_string() == current_version
        },
    } {
        return Ok(None);
    }

    Ok(Some(UpdatePackage {
        version: chosen.version,
        download_url: package.download_url(),
        sha256: package.sha256,
        size: package.size,
        cli_path: package.cli_path,
    }))
}

#[cfg(test)]
mod tests {
    use fig_util::{
        OLD_CLI_BINARY_NAME,
        OLD_PRODUCT_NAME,
    };

    use super::*;

    #[tokio::test]
    #[cfg(target_os = "macos")]
    async fn pull_test() {
        let index = pull(&Channel::Stable).await.unwrap();
        println!("{:#?}", index);
        assert!(!index.supported.is_empty());
        assert!(!index.versions.is_empty());
    }

    #[tokio::test]
    #[cfg(target_os = "macos")]
    #[ignore = "New index format not used yet"]
    async fn check_test() {
        check_for_updates(Channel::Stable, Os::Macos, Variant::Full, false)
            .await
            .unwrap();
    }

    #[test]
    fn test_release_url() {
        println!("{}", *release_url());
        println!("{:#?}", *release_url());
    }

    #[test]
    fn index_serde_test() {
        let json_str = serde_json::json!({
            "supported": [
                {
                    "kind": "dmg",
                    "os": "macos",
                    "architecture": "universal",
                    "variant": "full",
                    "fileType": "dmg"
                },
                {
                    "kind": "deb",
                    "os": "linux",
                    "architecture": "x86_64",
                    "variant": "headless",
                    "fileType": "tar_zst"
                }
            ],
            "versions": [
                {
                    "version": "0.7.0",
                    "rollout": null,
                    "packages": [
                        {
                            "kind": "dmg",
                            "architecture": "universal",
                            "variant": "full",
                            "download": format!("0.7.0/{OLD_PRODUCT_NAME}.dmg"),
                            "sha256": "4213d7649e4b1a2ec50adc0266d32d3e1e1f952ed6a863c28d7538190dc92472",
                            "size": 82975504
                        }
                    ]
                },
                {
                    "version": "0.15.3",
                    "packages": [
                        {
                            "kind": "dmg",
                            "architecture": "universal",
                            "variant": "full",
                            "download": format!("0.15.3/{OLD_PRODUCT_NAME}.dmg"),
                            "sha256": "87a311e493bb2b0e68a1b4b5d267c79628d23c1e39b0a62d1a80b0c2352f80a2",
                            "size": 88174538,
                            "cliPath": format!("Contents/MacOS/{OLD_CLI_BINARY_NAME}")
                        }
                    ]
                },
                {
                    "version": "1.0.0",
                    "packages": [
                        {
                            "kind": "deb",
                            "fileType": "dmg",
                            "os": "macos",
                            "architecture": "universal",
                            "variant": "full",
                            "download": "1.0.0/Q.dmg",
                            "sha256": "87a311e493bb2b0e68a1b4b5d267c79628d23c1e39b0a62d1a80b0c2352f80a2",
                            "size": 88174538,
                            "cliPath": format!("Contents/MacOS/{OLD_CLI_BINARY_NAME}"),
                        },
                        {
                            "kind": "deb",
                            "fileType": "tar_zst",
                            "os": "linux",
                            "architecture": "x86_64",
                            "variant": "headless",
                            "download": "1.0.0/q-x86_64-linux.tar.zst",
                            "sha256": "5a6abea56bfa91bd58d49fe40322058d0efea825f7e19f7fb7db1c204ae625b6",
                            "size": 76836772,
                        }
                    ]
                },
                {
                    "version": "2.0.0",
                    "packages": [
                        {
                            // random values to ensure forward compat
                            "kind": "abc",
                            "fileType": "abc",
                            "os": "abc",
                            "architecture": "abc",
                            "variant": "abc",
                            "download": "abc",
                            "sha256": "abc",
                            "size": 123,
                            "cliPath": "abc",
                            "otherField": "abc"
                        }
                    ]
                }
            ]
        })
        .to_string();

        let index = serde_json::from_str::<Index>(&json_str).unwrap();
        println!("{:#?}", index);

        assert_eq!(index.supported.len(), 2);
        assert_eq!(index.supported[0], Support {
            architecture: PackageArchitecture::Universal,
            variant: Variant::Full,
            os: Some(Os::Macos),
            file_type: Some(FileType::Dmg),
        });

        assert_eq!(index.versions.len(), 4);

        // check the 1.0.0 entry matches
        assert_eq!(index.versions[2], RemoteVersion {
            version: Version::new(1, 0, 0),
            rollout: None,
            packages: vec![
                Package {
                    architecture: PackageArchitecture::Universal,
                    variant: Variant::Full,
                    os: Some(Os::Macos),
                    file_type: Some(FileType::Dmg),
                    download: "1.0.0/Q.dmg".into(),
                    sha256: "87a311e493bb2b0e68a1b4b5d267c79628d23c1e39b0a62d1a80b0c2352f80a2".into(),
                    size: 88174538,
                    cli_path: Some(format!("Contents/MacOS/{OLD_CLI_BINARY_NAME}")),
                },
                Package {
                    architecture: PackageArchitecture::X86_64,
                    variant: Variant::Minimal,
                    os: Some(Os::Linux),
                    file_type: Some(FileType::TarZst),
                    download: "1.0.0/q-x86_64-linux.tar.zst".into(),
                    sha256: "5a6abea56bfa91bd58d49fe40322058d0efea825f7e19f7fb7db1c204ae625b6".into(),
                    size: 76836772,
                    cli_path: None,
                }
            ],
        });
    }
}
