mod install_method;

use anyhow::Result;
use serde::{ser::SerializeMap, Serialize};
use std::collections::HashMap;

const TELEMETRY_URL: &str = "https://tel.withfig.com/track";

/// An event that can be sent to the telemetry service
#[derive(Debug, Clone)]
pub struct SegmentEvent {
    user_id: String,
    event: String,
    properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Cli,
    Daemon,
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::Cli => f.write_str("cli"),
            Source::Daemon => f.write_str("daemon"),
        }
    }
}

impl SegmentEvent {
    /// Create a new SegmentEvent
    pub fn new(event: impl Into<String>) -> Result<Self> {
        // Check that telemetry is not disabled
        let telemetry_disabled = fig_settings::settings::get_value("telemetry.disabled")
            .ok()
            .flatten()
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if telemetry_disabled {
            return Err(anyhow::anyhow!("Telemetry is disabled"));
        }

        let user_id = fig_auth::get_default("uuid")?;

        Ok(SegmentEvent {
            user_id,
            event: event.into(),
            properties: HashMap::new(),
        })
    }

    /// Add the default properties to the event
    ///
    /// This includes email, version, build, install method, and source
    pub fn add_default_properties(&mut self, source: Source) -> Result<&mut Self> {
        if let Some(email) = fig_auth::get_email() {
            self.properties.insert("email".into(), email);
        }

        if let Ok(defaults_version) = fig_auth::get_default("versionAtPreviousLaunch") {
            if let Some((version, build)) = defaults_version.split_once(',') {
                self.properties.insert("version".into(), version.into());
                self.properties.insert("build".into(), build.into());
            }
        }

        self.properties.insert("source".into(), source.to_string());

        self.properties.insert(
            "install_method".into(),
            install_method::get_install_method().to_string(),
        );

        Ok(self)
    }

    /// Add a property to the event
    pub fn add_property(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    pub async fn send_event(&self) -> reqwest::Result<reqwest::Response> {
        reqwest::Client::new()
            .post(TELEMETRY_URL)
            .header("Content-Type", "application/json")
            .json(&self)
            .send()
            .await
    }
}

impl Serialize for SegmentEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_map(Some(2 + self.properties.len()))?;
        state.serialize_entry("userId", &self.user_id)?;
        state.serialize_entry("event", &self.event)?;
        for (key, value) in &self.properties {
            state.serialize_entry(&format!("prop_{}", key), value)?;
        }
        state.end()
    }
}
