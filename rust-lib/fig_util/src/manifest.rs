use std::fmt::Display;
use std::str::FromStr;

use cfg_if::cfg_if;
use once_cell::sync::Lazy;
use serde::{
    Deserialize,
    Deserializer,
};
use strum::EnumString;

#[derive(Deserialize)]
pub struct Manifest {
    #[serde(deserialize_with = "deser_enum_other")]
    pub managed_by: ManagedBy,
    #[serde(deserialize_with = "deser_enum_other")]
    pub variant: Variant,
    #[serde(deserialize_with = "deser_enum_other")]
    pub kind: Kind,
    #[serde(deserialize_with = "deser_enum_other")]
    pub default_channel: Channel,
    pub packaged_at: String,
    pub packaged_by: String,
}

#[derive(EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum ManagedBy {
    Apt,
    Dnf,
    Pacman,
    #[strum(default)]
    Other(String),
}

#[derive(EnumString, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Variant {
    Online,
    Offline,
    Full,
    Headless,
    #[strum(default)]
    Other(String),
}

#[derive(EnumString, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Kind {
    WindowsInstaller,
    WindowsBundle,
    Dmg,
    Tar,
    Deb,
    Rpm,
    #[strum(default)]
    Other(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Deserialize)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Channel {
    Nightly,
    Qa,
    Beta,
    Stable,
}

impl Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Channel::Nightly => write!(f, "Nightly"),
            Channel::Qa => write!(f, "QA"),
            Channel::Beta => write!(f, "Beta"),
            Channel::Stable => write!(f, "Stable"),
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

static CACHED: Lazy<Option<Manifest>> = Lazy::new(|| {
    cfg_if! {
        if #[cfg(unix)] {
            let text = match std::fs::read_to_string(crate::directories::manifest_path().unwrap()) {
                Ok(s) => s,
                Err(err) => {
                    tracing::warn!("Failed reading build manifest: {err}");
                    return None;
                },
            };
            match serde_json::from_str(&text) {
                Ok(s) => Some(s),
                Err(err) => {
                    tracing::warn!("Failed deserializing build manifest: {err:?}");
                    None
                },
            }
        } else {
            None
        }
    }
});

/// Returns the manifest, reading and parsing it if necessary
pub fn manifest() -> &'static Option<Manifest> {
    &CACHED
}

/// Checks if this is a full build according to the manifest.
/// Note that this does not guarantee the value of is_headless
pub fn is_full() -> bool {
    cfg_if! {
        if #[cfg(target_os = "macos")] {
            true
        } else if #[cfg(unix)] {
            matches!(
                manifest(),
                Some(Manifest {
                    variant: Variant::Full,
                    ..
                })
            )
        } else if #[cfg(windows)] {
            true
        }
    }
}

/// Checks if this is a headless build according to the manifest.
/// Note that this does not guarantee the value of is_full
pub fn is_headless() -> bool {
    cfg_if! {
        if #[cfg(target_os = "macos")] {
            false
        } else if #[cfg(unix)] {
            matches!(
                manifest(),
                Some(Manifest {
                    variant: Variant::Headless,
                    ..
                })
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
