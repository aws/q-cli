use cfg_if::cfg_if;
use once_cell::sync::Lazy;
use serde::{
    Deserialize,
    Deserializer,
};

#[derive(Deserialize)]
pub struct Manifest {
    #[serde(deserialize_with = "deser_managed")]
    pub managed_by: ManagedBy,
    #[serde(deserialize_with = "deser_variant")]
    pub variant: Variant,
    pub packaged_at: String,
    pub packaged_by: String,
    pub version: String,
}

pub enum ManagedBy {
    Apt,
    Dnf,
    Pacman,
    Other(String),
}

pub enum Variant {
    Full,
    Headless,
    Other(String),
}

fn deser_managed<'de, D>(deserializer: D) -> Result<ManagedBy, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(match <&str as Deserialize<'de>>::deserialize(deserializer)? {
        "apt" => ManagedBy::Apt,
        "dnf" => ManagedBy::Dnf,
        "pacman" => ManagedBy::Pacman,
        other => ManagedBy::Other(other.to_string()),
    })
}

fn deser_variant<'de, D>(deserializer: D) -> Result<Variant, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(match <&str as Deserialize<'de>>::deserialize(deserializer)? {
        "full" => Variant::Full,
        "headless" => Variant::Headless,
        other => Variant::Other(other.to_string()),
    })
}

static CACHED: Lazy<Option<Manifest>> = Lazy::new(|| {
    cfg_if! {
        if #[cfg(all(unix, not(target_os = "macos")))] {
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

#[cfg(target_os = "macos")]
static MACOS_VERSION: Lazy<Option<String>> = Lazy::new(|| {
    let version = option_env!("VERSION");
    let build = option_env!("BUILD");
    match (version, build) {
        (Some(version), Some(build)) => Some(format!("{version}+{build}")),
        (Some(version), None) => Some(version.into()),
        _ => None,
    }
});

#[cfg(target_os = "windows")]
static WINDOWS_VERSION: Lazy<Option<String>> = Lazy::new(|| {
    let output = std::process::Command::new("fig_desktop.exe").arg("--version").output();
    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let version = stdout.replace("fig_desktop", "").trim().to_owned();
            Some(version)
        },
        Err(_) => None,
    }
});

/// Gets the version from the manifest
pub fn version() -> Option<&'static str> {
    match manifest() {
        Some(manifest) => Some(&manifest.version),
        None => {
            cfg_if! {
                if #[cfg(target_os = "macos")] {
                    MACOS_VERSION.as_deref()
                } else if #[cfg(target_os = "windows")] {
                    // TODO(mia): add actual manifest version for windows
                    WINDOWS_VERSION.as_deref()
                } else {
                    None
                }
            }
        },
    }
}
