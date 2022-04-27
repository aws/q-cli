use fig_proto::fig::{TelemetryAliasRequest, TelemetryIdentifyRequest, TelemetryTrackRequest};
use fig_telemetry::{emit_alias, emit_identify, emit_track, TrackEvent, TrackSource};

use super::{ResponseKind, ResponseResult};

pub async fn handle_alias_request(request: TelemetryAliasRequest, _: i64) -> ResponseResult {
    let user_id = request
        .user_id
        .ok_or_else(|| ResponseKind::Error("Empty user id".into()))?;

    emit_alias(user_id)
        .await
        .map_err(|e| ResponseKind::Error(format!("Failed to emit alias, {e}")))?;

    Ok(ResponseKind::Success)
}

pub async fn handle_identify_request(request: TelemetryIdentifyRequest, _: i64) -> ResponseResult {
    let traits: Vec<(&str, &str)> = request
        .traits
        .iter()
        .map(|t| (t.key.as_str(), t.value.as_str()))
        .collect();

    emit_identify(traits)
        .await
        .map_err(|e| ResponseKind::Error(format!("Failed to emit identify, {e}")))?;

    Ok(ResponseKind::Success)
}

pub async fn handle_track_request(request: TelemetryTrackRequest, _: i64) -> ResponseResult {
    let event = request
        .event
        .ok_or_else(|| ResponseKind::Error("Empty track event".into()))?;

    let properties: Vec<(&str, &str)> = request
        .properties
        .iter()
        .map(|p| (p.key.as_str(), p.value.as_str()))
        .collect();

    emit_track(TrackEvent::Other(event), TrackSource::App, properties)
        .await
        .map_err(|e| ResponseKind::Error(format!("Failed to emit track, {e}")))?;

    Ok(ResponseKind::Success)
}
