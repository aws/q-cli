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
    K: Into<String>,
    V: Into<Value>,
{
    if telemetry_is_disabled() {
        return Err(Error::TelemetryDisabled);
    }

    let mut props = crate::util::default_properties();
    props.insert("source".into(), source.to_string().into());
    props.extend(properties.into_iter().map(|(k, v)| (k.into(), v.into())));

    let mut body: Map<String, Value> = Map::new();
    body.insert("event".into(), event.to_string().into());
    body.insert("useUnprefixed".into(), true.into());
    body.insert("properties".into(), props.into());

    make_telemetry_request(TRACK_SUBDOMAIN, body).await
}
