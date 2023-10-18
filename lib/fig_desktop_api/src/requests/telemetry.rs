use fig_proto::fig::{
    TelemetryIdentifyRequest,
    TelemetryPageRequest,
    TelemetryTrackRequest,
};
// use fig_telemetry::{
//     emit_identify,
//     emit_page,
//     emit_track,
//     TrackEvent,
//     TrackEventType,
//     TrackSource,
// };
use serde_json::{
    Map,
    Value,
};

use super::{
    RequestResult,
    RequestResultImpl,
};

pub async fn handle_identify_request(request: TelemetryIdentifyRequest) -> RequestResult {
    let mut traits: Map<String, Value> = Default::default();

    if let Some(ref json_blob) = request.json_blob {
        match serde_json::from_str::<serde_json::Map<String, Value>>(json_blob) {
            Ok(props) => {
                traits.extend(props);
            },
            Err(err) => return Err(format!("Failed to decode json blob: {err}").into()),
        }
    }

    // emit_identify(traits)
    //     .await
    //     .map_err(|err| format!("Failed to emit identify: {err}"))?;

    RequestResult::success()
}

pub async fn handle_track_request(request: TelemetryTrackRequest) -> RequestResult {
    let _event: String = request.event.ok_or_else(|| "Empty track event".to_string())?;

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
            Err(err) => return Err(format!("Failed to decode json blob: {err}").into()),
        }
    }

    // let event = TrackEvent::new(
    //     TrackEventType::Other(event),
    //     TrackSource::Desktop,
    //     env!("CARGO_PKG_VERSION").into(),
    //     properties,
    // )
    // .with_namespace(request.namespace)
    // .with_namespace_id(request.namespace_id);

    // emit_track(event)
    //     .await
    //     .map_err(|err| format!("Failed to emit track: {err}"))?;

    RequestResult::success()
}

pub async fn handle_page_request(request: TelemetryPageRequest) -> RequestResult {
    let _properties = if let Some(ref json_blob) = request.json_blob {
        match serde_json::from_str::<serde_json::Map<String, Value>>(json_blob) {
            Ok(props) => props,
            Err(err) => return Err(format!("Failed to decode json blob: {err}").into()),
        }
    } else {
        Map::new()
    };

    // emit_page(
    //     request.category().into(),
    //     request.name().into(),
    //     TrackSource::Desktop,
    //     properties,
    // )
    // .await
    // .map_err(|err| format!("Failed to emit track: {err}"))?;

    RequestResult::success()
}
