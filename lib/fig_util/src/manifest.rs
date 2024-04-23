use std::fmt::Display;
use std::str::FromStr;

use cfg_if::cfg_if;
use once_cell::sync::Lazy;
use serde::{
    Deserialize,
    Deserializer,
    Serialize,
};
use strum::{
    Display,
    EnumString,
};

#[derive(Deserialize)]
pub struct Manifest {
    #[serde(deserialize_with = "deser_enum_other")]
    pub managed_by: ManagedBy,
    #[serde(deserialize_with = "deser_enum_other")]
    pub variant: Variant,
    #[serde(deserialize_with = "deser_enum_other")]
    pub default_channel: Channel,
    pub packaged_at: String,
    pub packaged_by: String,
}

#[derive(EnumString, Display)]
#[strum(serialize_all = "snake_case")]
pub enum ManagedBy {
    Apt,
    Dnf,
    Pacman,
    #[strum(default)]
    Other(String),
}

#[derive(EnumString, Display, Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Variant {
    Full,
    Minimal,
    #[strum(default)]
    Other(String),
}

#[derive(EnumString, Display, Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
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
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum FileType {
    Dmg,
    TarGz,
    TarXz,
    TarZst,
    Zip,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumString, Deserialize)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
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

static CACHED: Lazy<Manifest> = Lazy::new(|| Manifest {
    managed_by: ManagedBy::Other("aws".into()),
    variant: match option_env!("Q_BUILD_VARIANT")
        .map(|s| s.to_ascii_lowercase())
        .as_deref()
    {
        Some("minimal") => Variant::Minimal,
        _ => Variant::Full,
    },
    default_channel: Channel::Stable,
    packaged_at: "unknown".into(),
    packaged_by: "unknown".into(),
});

/// Returns the manifest, reading and parsing it if necessary
pub fn manifest() -> &'static Manifest {
    &CACHED
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
    use super::*;

    #[test]
    fn test_file_type_serialize_deserialize() {
        use serde_json::{
            from_str,
            to_string,
        };

        assert_eq!("\"dmg\"", to_string(&FileType::Dmg).unwrap());
        assert_eq!("\"tar_gz\"", to_string(&FileType::TarGz).unwrap());
        assert_eq!("\"tar_xz\"", to_string(&FileType::TarXz).unwrap());
        assert_eq!("\"tar_zst\"", to_string(&FileType::TarZst).unwrap());
        assert_eq!("\"zip\"", to_string(&FileType::Zip).unwrap());

        assert_eq!(FileType::Dmg, from_str("\"dmg\"").unwrap());
        assert_eq!(FileType::TarGz, from_str("\"tar_gz\"").unwrap());
        assert_eq!(FileType::TarXz, from_str("\"tar_xz\"").unwrap());
        assert_eq!(FileType::TarZst, from_str("\"tar_zst\"").unwrap());
        assert_eq!(FileType::Zip, from_str("\"zip\"").unwrap());
    }
}
