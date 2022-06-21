use anyhow::anyhow;
use fig_proto::fig::{
    TelemetryAliasRequest,
    TelemetryIdentifyRequest,
    TelemetryTrackRequest,
};
use fig_telemetry::{
    emit_alias,
    emit_identify,
    emit_track,
    TrackEvent,
    TrackSource,
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
    let traits: Vec<(&str, &str)> = request
        .traits
        .iter()
        .map(|t| (t.key.as_str(), t.value.as_str()))
        .collect();

    emit_identify(traits)
        .await
        .map_err(|e| anyhow!("Failed to emit identify, {e}"))?;

    RequestResult::success()
}

pub async fn handle_track_request(request: TelemetryTrackRequest) -> RequestResult {
    let event: String = request.event.ok_or_else(|| anyhow!("Empty track event"))?;

    let properties: Vec<(&str, &str)> = request
        .properties
        .iter()
        .map(|p| (p.key.as_str(), p.value.as_str()))
        .collect();

    emit_track(TrackEvent::Other(event), TrackSource::App, properties)
        .await
        .map_err(|e| anyhow!("Failed to emit track, {e}"))?;

    RequestResult::success()
}
