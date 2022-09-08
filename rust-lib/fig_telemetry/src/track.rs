use fig_proto::daemon::telemetry_emit_track_command::Source;
use fig_proto::daemon::TelemetryEmitTrackCommand;
use serde::{
    Deserialize,
    Serialize,
};
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackEventType {
    RanCommand,
    DoctorError,
    WorkflowSearchViewed,
    WorkflowExecuted,
    WorkflowCancelled,
    LaunchedApp,
    QuitApp,
    UninstallApp,
    UpdatedApp,
    TerminalSessionMetricsRecorded,
    DotfileLineCountsRecorded,
    Login,
    Logout,
    /// Prefer not using this directly and instead define an enum value, this is only for
    /// internal use by `fig_telemetry`
    Other(String),
}

impl std::fmt::Display for TrackEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::RanCommand => "Ran CLI command",
            Self::DoctorError => "Doctor Error",
            Self::WorkflowSearchViewed => "Workflow Search Viewed",
            Self::WorkflowExecuted => "Workflow Executed",
            Self::WorkflowCancelled => "Workflow Cancelled",
            Self::LaunchedApp => "Launched App",
            Self::QuitApp => "Quit App",
            Self::UninstallApp => "Uninstall App",
            Self::UpdatedApp => "Updated App",
            Self::TerminalSessionMetricsRecorded => "Terminal Session Metrics Recorded",
            Self::DotfileLineCountsRecorded => "Dotfile Line Counts Recorded",
            Self::Login => "login",
            Self::Logout => "User Logged Out",
            Self::Other(s) => s,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackSource {
    Desktop,
    Cli,
    Daemon,
}

impl std::fmt::Display for TrackSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Desktop => "desktop",
            Self::Cli => "cli",
            Self::Daemon => "daemon",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackEvent {
    pub event: TrackEventType,
    pub source: TrackSource,
    pub source_version: Option<String>,
    pub properties: Map<String, Value>,
}

impl TrackEvent {
    pub fn new<I, K, V>(event: TrackEventType, source: TrackSource, source_version: String, properties: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<Value>,
    {
        TrackEvent {
            event,
            source,
            source_version: Some(source_version),
            properties: properties.into_iter().map(|(k, v)| (k.into(), v.into())).collect(),
        }
    }

    pub fn to_map(&self, props: &Map<String, Value>) -> Map<String, Value> {
        let mut res: Map<String, Value> = Map::new();

        let mut props = props.clone();
        props.insert("event_origination_source".into(), self.source.to_string().into());
        if let Some(ref source_version) = self.source_version {
            props.insert(format!("{}_version", self.source), source_version.clone().into());
        }
        props.extend(self.properties.clone());

        res.insert("event".into(), self.event.to_string().into());
        res.insert("properties".into(), props.into());

        res
    }
}

impl From<&TelemetryEmitTrackCommand> for TrackEvent {
    fn from(command: &TelemetryEmitTrackCommand) -> Self {
        let event = TrackEventType::Other(command.event.clone());

        let properties: Map<String, serde_json::Value> = command
            .properties
            .iter()
            .map(|(key, value)| (key.clone(), value.clone().into()))
            .collect();

        let source = match Source::from_i32(command.source.unwrap_or_default()).unwrap_or_default() {
            Source::Desktop => TrackSource::Desktop,
            Source::Cli => TrackSource::Cli,
            Source::Daemon => TrackSource::Daemon,
        };

        let source_version = command.source_version.clone();

        TrackEvent {
            event,
            source,
            source_version,
            properties,
        }
    }
}

pub async fn emit_track(event: TrackEvent) -> Result<(), Error> {
    if telemetry_is_disabled() {
        return Err(Error::TelemetryDisabled);
    }

    let props = crate::util::default_properties();
    let mut body = event.to_map(&props);
    body.insert("useUnprefixed".into(), true.into());

    make_telemetry_request(TRACK_SUBDOMAIN, body).await
}

pub async fn emit_tracks(events: Vec<TrackEvent>) -> Result<(), Error> {
    if telemetry_is_disabled() {
        return Err(Error::TelemetryDisabled);
    }

    let props = crate::util::default_properties();
    let events: Vec<Value> = events.into_iter().map(|e| e.to_map(&props).into()).collect();

    let mut body: Map<String, Value> = Map::new();
    body.insert("events".into(), events.into());
    body.insert("useUnprefixed".into(), true.into());

    make_telemetry_request(TRACK_SUBDOMAIN, body).await
}
