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
    /// This includes email
    pub fn add_default_properties(&mut self) -> Result<&mut Self> {
        self.properties
            .insert("email".to_string(), fig_auth::get_default("email")?);

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
