use std::fmt::Display;

use fig_util::get_system_id;
use serde_json::{
    Map,
    Value,
};

use crate::util::{
    make_telemetry_request,
    telemetry_is_disabled,
};
use crate::{
    Error,
    TRACK_SUBDOMAIN,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrackEvent {
    RanCommand,
    DoctorError,
    WorkflowSearchViewed,
    WorkflowExecuted,
    LaunchedApp,
    QuitApp,
    UninstallApp,
    UpdatedApp,
    /// Prefer not using this directly and instead define an enum value, this is only for
    /// internal use by `fig_telemetry`
    Other(String),
}

impl std::fmt::Display for TrackEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::RanCommand => "Ran CLI command",
            Self::DoctorError => "Doctor Error",
            Self::WorkflowSearchViewed => "Workflow Search Viewed",
            Self::WorkflowExecuted => "Workflow Executed",
            Self::LaunchedApp => "Launched App",
            Self::QuitApp => "Quit App",
            Self::UninstallApp => "Uninstall App",
            Self::UpdatedApp => "Updated App",
            Self::Other(s) => s,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackSource {
    App,
    Cli,
    Daemon,
}

impl std::fmt::Display for TrackSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::App => "app",
            Self::Cli => "cli",
            Self::Daemon => "daemon",
        })
    }
}

pub async fn emit_track<'a, I, K, V>(event: TrackEvent, source: TrackSource, properties: I) -> Result<(), Error>
where
    I: IntoIterator<Item = (K, V)>,
    K: Display,
    V: Into<Value>,
{
    if telemetry_is_disabled() {
        return Err(Error::TelemetryDisabled);
    }

    // Initial properties
    let mut track: Map<String, Value> = Map::new();
    track.insert("event".into(), event.to_string().into());

    // Default properties
    if let Some(email) = fig_auth::get_email() {
        if let Some(domain) = email.split('@').last() {
            track.insert("prop_domain".into(), domain.into());
        }
        track.insert("prop_email".into(), email.into());
    }

    #[cfg(target_os = "macos")]
    if let Ok(version) = fig_auth::get_default("versionAtPreviousLaunch") {
        if let Some((version, build)) = version.split_once(',') {
            track.insert("prop_version".into(), version.into());
            track.insert("prop_build".into(), build.into());
        }
    }

    track.insert("prop_source".into(), source.to_string().into());

    track.insert(
        "prop_install_method".into(),
        crate::install_method::get_install_method().to_string().into(),
    );

    if let Ok(device_id) = get_system_id() {
        track.insert("prop_device_id".into(), device_id.into());
    }

    track.insert("prop_device_os".into(), std::env::consts::OS.into());
    track.insert("prop_device_arch".into(), std::env::consts::ARCH.into());

    // Given properties
    for (key, value) in properties.into_iter() {
        track.insert(format!("prop_{key}"), value.into());
    }

    make_telemetry_request(TRACK_SUBDOMAIN, track).await
}
