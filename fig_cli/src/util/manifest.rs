use cfg_if::cfg_if;
#[cfg(target_os = "linux")]
use fig_util::directories;
use once_cell::sync::Lazy;
use serde::{
    Deserialize,
    Deserializer,
};
#[cfg(target_os = "linux")]
use tracing::warn;

#[derive(Deserialize)]
pub struct Manifest {
    #[serde(deserialize_with = "deser_managed")]
    pub managed_by: ManagedBy,
    #[serde(deserialize_with = "deser_variant")]
    pub variant: Variant,
    pub packaged_at: String,
    pub packaged_by: String,
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
        if #[cfg(target_os = "linux")] {
            let text = match std::fs::read_to_string(directories::manifest_path()) {
                Ok(s) => s,
                Err(err) => {
                    warn!("Failed reading build manifest: {err}");
                    return None;
                },
            };
            match serde_json::from_str(&text) {
                Ok(s) => Some(s),
                Err(err) => {
                    warn!("Failed deserializing build manifest: {err:?}");
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

/// Checks if this is a full build according to the manifest. Note that this does not guarantee the
/// value of is_headless
pub fn is_full() -> bool {
    matches!(
        manifest(),
        Some(Manifest {
            variant: Variant::Full,
            ..
        })
    )
}

/// Checks if this is a headless build according to the manifest. Note that this does not guarantee
/// the value of is_full
pub fn is_headless() -> bool {
    matches!(
        manifest(),
        Some(Manifest {
            variant: Variant::Headless,
            ..
        })
    )
}
