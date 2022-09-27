use anyhow::{
    anyhow,
    bail,
};
use fig_proto::fig::aggregate_session_metric_action_request::{
    Action,
    Increment,
};
use fig_proto::fig::{
    AggregateSessionMetricActionRequest,
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
    TrackEventType,
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
use crate::figterm::FigtermState;

pub async fn handle_alias_request(request: TelemetryAliasRequest) -> RequestResult {
    let user_id = request.user_id.ok_or_else(|| anyhow!("Empty user id"))?;

    emit_alias(user_id)
        .await
        .map_err(|err| anyhow!("Failed to emit alias: {err}"))?;

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
            Err(err) => bail!("Failed to decode json blob: {err}"),
        }
    }

    emit_identify(traits)
        .await
        .map_err(|err| anyhow!("Failed to emit identify: {err}"))?;

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
            Err(err) => bail!("Failed to decode json blob: {err}"),
        }
    }

    emit_track(TrackEvent::new(
        TrackEventType::Other(event),
        TrackSource::Desktop,
        env!("CARGO_PKG_VERSION").into(),
        properties,
    ))
    .await
    .map_err(|err| anyhow!("Failed to emit track: {err}"))?;

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
        TrackSource::Desktop,
        properties,
    )
    .await
    .map_err(|err| anyhow!("Failed to emit track: {err}"))?;

    RequestResult::success()
}

pub fn handle_aggregate_session_metric_action_request(
    request: AggregateSessionMetricActionRequest,
    state: &FigtermState,
) -> RequestResult {
    if let Some(result) = state.with_most_recent(|session| {
        if let Some(ref mut metrics) = session.current_session_metrics {
            if let Some(action) = request.action {
                match action {
                    Action::Increment(Increment { field, amount }) => {
                        if field == "num_popups" {
                            metrics.num_popups += amount.unwrap_or(1);
                        } else {
                            bail!("Unknown field: {field}");
                        }
                    },
                };
            }
        }
        Ok(())
    }) {
        result?;
    }

    RequestResult::success()
}
