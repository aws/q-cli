use anyhow::{
    anyhow,
    bail,
};
use fig_proto::fig::{
    TelemetryAliasRequest,
    TelemetryIdentifyRequest,
    TelemetryPageRequest,
    TelemetryTrackRequest,
};
use fig_telemetry::{
    emit_alias,
    emit_identify,
    emit_page,
    emit_track,
    TrackEvent,
    TrackSource,
};
use serde_json::{
    Map,
    Value,
};

use super::{
    RequestResult,
    RequestResultImpl,
};

pub async fn handle_alias_request(request: TelemetryAliasRequest) -> RequestResult {
    let user_id = request.user_id.ok_or_else(|| anyhow!("Empty user id"))?;

    emit_alias(user_id)
        .await
        .map_err(|e| anyhow!("Failed to emit alias, {e}"))?;

    RequestResult::success()
}

pub async fn handle_identify_request(request: TelemetryIdentifyRequest) -> RequestResult {
    #[allow(deprecated)]
    let mut traits: Map<String, Value> = request
        .traits
        .iter()
        .map(|t| (t.key.as_str().into(), t.value.as_str().into()))
        .collect();

    if let Some(ref json_blob) = request.json_blob {
        match serde_json::from_str::<serde_json::Map<String, Value>>(json_blob) {
            Ok(props) => {
                traits.extend(props);
            },
            Err(err) => {
                bail!("Failed to decode json blob: {err}");
            },
        }
    }

    emit_identify(traits)
        .await
        .map_err(|e| anyhow!("Failed to emit identify, {e}"))?;

    RequestResult::success()
}

pub async fn handle_track_request(request: TelemetryTrackRequest) -> RequestResult {
    let event: String = request.event.ok_or_else(|| anyhow!("Empty track event"))?;

    #[allow(deprecated)]
    let mut properties: Map<String, Value> = request
        .properties
        .iter()
        .map(|p| (p.key.as_str().into(), p.value.as_str().into()))
        .collect();

    if let Some(ref json_blob) = request.json_blob {
        match serde_json::from_str::<serde_json::Map<String, Value>>(json_blob) {
            Ok(props) => {
                properties.extend(props);
            },
            Err(err) => {
                bail!("Failed to decode json blob: {err}");
            },
        }
    }

    emit_track(TrackEvent::Other(event), TrackSource::App, properties)
        .await
        .map_err(|e| anyhow!("Failed to emit track, {e}"))?;

    RequestResult::success()
}

pub async fn handle_page_request(request: TelemetryPageRequest) -> RequestResult {
    let properties = if let Some(ref json_blob) = request.json_blob {
        match serde_json::from_str::<serde_json::Map<String, Value>>(json_blob) {
            Ok(props) => props,
            Err(err) => {
                bail!("Failed to decode json blob: {err}");
            },
        }
    } else {
        Map::new()
    };

    emit_page(
        request.category().into(),
        request.name().into(),
        TrackSource::App,
        properties,
    )
    .await
    .map_err(|e| anyhow!("Failed to emit track, {e}"))?;

    RequestResult::success()
}
