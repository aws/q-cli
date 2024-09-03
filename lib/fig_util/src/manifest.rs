use std::fmt::Display;
use std::str::FromStr;
use std::sync::OnceLock;

use cfg_if::cfg_if;
use serde::{
    Deserialize,
    Deserializer,
    Serialize,
};
use strum::{
    Display,
    EnumString,
};

use crate::build::{
    PACKAGED_AS,
    TARGET_TRIPLE,
};
use crate::consts::build::VARIANT;

#[derive(Deserialize)]
pub struct Manifest {
    #[serde(deserialize_with = "deser_enum_other")]
    pub managed_by: ManagedBy,
    #[serde(deserialize_with = "deser_enum_other")]
    pub target_triple: TargetTriple,
    #[serde(deserialize_with = "deser_enum_other")]
    pub variant: Variant,
    #[serde(deserialize_with = "deser_enum_other")]
    pub default_channel: Channel,
    pub packaged_as: PackagedAs,
    pub packaged_at: String,
    pub packaged_by: String,
}

#[derive(EnumString, Display, Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ManagedBy {
    None,
    #[strum(default)]
    Other(String),
}

/// The target triplet, describes a platform on which the project is build for. Note that this also
/// includes "fake" targets like `universal-apple-darwin` as provided by [Tauri](https://tauri.app/v1/guides/building/macos/#binary-targets)
#[derive(Deserialize, Serialize, PartialEq, Eq, EnumString, Debug, Display)]
pub enum TargetTriple {
    #[serde(rename = "universal-apple-darwin")]
    #[strum(serialize = "universal-apple-darwin")]
    UniversalAppleDarwin,
    #[serde(rename = "x86_64-unknown-linux-gnu")]
    #[strum(serialize = "x86_64-unknown-linux-gnu")]
    X86_64UnknownLinuxGnu,
    #[serde(rename = "x86_64-unknown-linux-musl")]
    #[strum(serialize = "x86_64-unknown-linux-musl")]
    X86_64UnknownLinuxMusl,
    #[serde(rename = "aarch64-unknown-linux-gnu")]
    #[strum(serialize = "aarch64-unknown-linux-gnu")]
    AArch64UnknownLinuxGnu,
    #[serde(rename = "aarch64-unknown-linux-musl")]
    #[strum(serialize = "aarch64-unknown-linux-musl")]
    AArch64UnknownLinuxMusl,
    #[strum(default)]
    Other(String),
}

impl TargetTriple {
    const fn from_system() -> Self {
        cfg_if! {
            if #[cfg(target_os = "macos")] {
                TargetTriple::UniversalAppleDarwin
            } else if #[cfg(all(target_os = "linux", target_env = "gnu", target_arch = "x86_64"))] {
                TargetTriple::X86_64UnknownLinuxGnu
            } else if #[cfg(all(target_os = "linux", target_env = "gnu", target_arch = "aarch64"))] {
                TargetTriple::AArch64UnknownLinuxGnu
            } else if #[cfg(all(target_os = "linux", target_env = "musl", target_arch = "x86_64"))] {
                TargetTriple::X86_64UnknownLinuxMusl
            } else if #[cfg(all(target_os = "linux", target_env = "musl", target_arch = "aarch64"))] {
                TargetTriple::AArch64UnknownLinuxMusl
            } else {
                compile_error!("unknown target")
            }
        }
    }
}

#[derive(EnumString, Display, Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum Variant {
    Full,
    #[serde(alias = "headless")]
    #[strum(to_string = "minimal", serialize = "headless")]
    Minimal,
    #[strum(default)]
    Other(String),
}

#[derive(EnumString, Display, Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum Os {
    Macos,
    Linux,
    #[strum(default)]
    Other(String),
}

impl Os {
    pub fn current() -> Self {
        match std::env::consts::OS {
            "macos" => Os::Macos,
            "linux" => Os::Linux,
            _ => panic!("Unsupported OS: {}", std::env::consts::OS),
        }
    }

    pub fn is_current_os(&self) -> bool {
        self == &Os::current()
    }
}

#[derive(EnumString, Display, Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum FileType {
    Dmg,
    TarGz,
    TarXz,
    TarZst,
    Zip,
    AppImage,
    #[strum(default)]
    Other(String),
}

impl FileType {
    pub const fn from_system() -> Self {
        cfg_if! {
            if #[cfg(target_os = "macos")] {
                FileType::Dmg
            } else if #[cfg(target_os = "linux")] {
                FileType::TarZst
            } else {
                compile_error!("unknown architecture")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumString, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum Channel {
    Stable,
    Beta,
    Qa,
    Nightly,
}

impl Channel {
    pub fn all() -> &'static [Self] {
        &[Channel::Stable, Channel::Beta, Channel::Qa, Channel::Nightly]
    }

    pub fn id(&self) -> &'static str {
        match self {
            Channel::Stable => "stable",
            Channel::Beta => "beta",
            Channel::Qa => "qa",
            Channel::Nightly => "nightly",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Channel::Stable => "Stable",
            Channel::Beta => "Beta",
            Channel::Qa => "QA",
            Channel::Nightly => "Nightly",
        }
    }
}

impl Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            f.write_str(self.name())
        } else {
            f.write_str(self.id())
        }
    }
}

/// How the application was packaged.
///
/// Note this is separate from distribution. For example, the app may be
/// packaged as a `.dmg` file but installed through public download links
/// or through toolbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display, EnumString, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum PackagedAs {
    /// Apple Disk Image, strictly for macOS.
    Dmg,
    /// AppImage, a universal installer for Linux distributions.
    AppImage,
    /// The deb format, for Debian-based Linux distributions.
    Deb,
    /// No packaging/bundling method was used. This would fit instances where
    /// the app binaries were distributed directly (e.g., directly within an
    /// archive format like `.zip`).
    None,
}

fn deser_enum_other<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: Display,
{
    match T::from_str(<&str as Deserialize<'de>>::deserialize(deserializer)?) {
        Ok(s) => Ok(s),
        Err(err) => Err(serde::de::Error::custom(err)),
    }
}

/// Returns the manifest, reading and parsing it if necessary
pub fn manifest() -> &'static Manifest {
    static CACHED: OnceLock<Manifest> = OnceLock::new();
    CACHED.get_or_init(|| Manifest {
        managed_by: ManagedBy::None,
        target_triple: match TARGET_TRIPLE {
            Some(target) => TargetTriple::from_str(target).expect("parsing target triple should not fail"),
            _ => TargetTriple::from_system(),
        },
        variant: match VARIANT.map(|s| s.to_ascii_lowercase()).as_deref() {
            Some("minimal") => Variant::Minimal,
            _ => Variant::Full,
        },
        packaged_as: match PACKAGED_AS {
            Some(packaged_as) => packaged_as.parse().expect("parsing PackagedAs should not fail"),
            None => PackagedAs::None,
        },
        default_channel: Channel::Stable,
        packaged_at: "unknown".into(),
        packaged_by: "unknown".into(),
    })
}

/// Checks if this is a full build according to the manifest.
/// Note that this does not guarantee the value of is_minimal
pub fn is_full() -> bool {
    cfg_if! {
        if #[cfg(target_os = "macos")] {
            true
        } else if #[cfg(unix)] {
            matches!(
                manifest(),
                Manifest {
                    variant: Variant::Full,
                    ..
                }
            )
        } else if #[cfg(windows)] {
            true
        }
    }
}

/// Checks if this is a minimal build according to the manifest.
/// Note that this does not guarantee the value of is_full
pub fn is_minimal() -> bool {
    cfg_if! {
        if #[cfg(target_os = "macos")] {
            false
        } else if #[cfg(unix)] {
            matches!(
                manifest(),
                Manifest {
                    variant: Variant::Minimal,
                    ..
                }
            )
        } else if #[cfg(windows)] {
            false
        }
    }
}

/// Gets the version from the manifest
#[deprecated = "versions are unified, use env!(\"CARGO_PKG_VERSION\")"]
pub fn version() -> Option<&'static str> {
    Some(env!("CARGO_PKG_VERSION"))
}

#[cfg(test)]
mod tests {
    use serde_json::{
        from_str,
        to_string,
    };

    use super::*;

    macro_rules! test_ser_deser {
        ($ty:ident, $variant:expr, $text:expr) => {
            let quoted = format!("\"{}\"", $text);
            assert_eq!(quoted, to_string(&$variant).unwrap());
            assert_eq!($variant, from_str(&quoted).unwrap());
            assert_eq!($variant, $ty::from_str($text).unwrap());
            assert_eq!($text, $variant.to_string());
        };
    }

    #[test]
    fn test_target_triple_serialize_deserialize() {
        test_ser_deser!(
            TargetTriple,
            TargetTriple::UniversalAppleDarwin,
            "universal-apple-darwin"
        );
        test_ser_deser!(
            TargetTriple,
            TargetTriple::X86_64UnknownLinuxGnu,
            "x86_64-unknown-linux-gnu"
        );
        test_ser_deser!(
            TargetTriple,
            TargetTriple::AArch64UnknownLinuxGnu,
            "aarch64-unknown-linux-gnu"
        );
        test_ser_deser!(
            TargetTriple,
            TargetTriple::X86_64UnknownLinuxMusl,
            "x86_64-unknown-linux-musl"
        );
        test_ser_deser!(
            TargetTriple,
            TargetTriple::AArch64UnknownLinuxMusl,
            "aarch64-unknown-linux-musl"
        );
    }

    #[test]
    fn test_file_type_serialize_deserialize() {
        test_ser_deser!(FileType, FileType::Dmg, "dmg");
        test_ser_deser!(FileType, FileType::TarGz, "tarGz");
        test_ser_deser!(FileType, FileType::TarXz, "tarXz");
        test_ser_deser!(FileType, FileType::TarZst, "tarZst");
        test_ser_deser!(FileType, FileType::Zip, "zip");
        test_ser_deser!(FileType, FileType::AppImage, "appImage");
    }

    #[test]
    fn test_managed_by_serialize_deserialize() {
        test_ser_deser!(ManagedBy, ManagedBy::None, "none");
    }

    #[test]
    fn test_variant_serialize_deserialize() {
        test_ser_deser!(Variant, Variant::Full, "full");
        test_ser_deser!(Variant, Variant::Minimal, "minimal");

        // headless is a special case that should deserialize to Minimal
        assert_eq!(Variant::Minimal, from_str("\"headless\"").unwrap());
        assert_eq!(Variant::Minimal, Variant::from_str("headless").unwrap());
    }

    #[test]
    fn test_channel_serialize_deserialize() {
        test_ser_deser!(Channel, Channel::Stable, "stable");
        test_ser_deser!(Channel, Channel::Beta, "beta");
        test_ser_deser!(Channel, Channel::Qa, "qa");
        test_ser_deser!(Channel, Channel::Nightly, "nightly");
    }

    #[test]
    fn test_packaged_as_serialize_deserialize() {
        test_ser_deser!(PackagedAs, PackagedAs::Dmg, "dmg");
        test_ser_deser!(PackagedAs, PackagedAs::AppImage, "appImage");
        test_ser_deser!(PackagedAs, PackagedAs::Deb, "deb");
        test_ser_deser!(PackagedAs, PackagedAs::None, "none");
    }
}
