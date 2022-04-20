use fig_proto::fig::{TelemetryAliasRequest, TelemetryIdentifyRequest, TelemetryTrackRequest};

use super::{ResponseKind, ResponseResult};

pub fn handle_alias_request(request: TelemetryAliasRequest, _: i64) -> ResponseResult {}

pub fn handle_track_request(request: TelemetryTrackRequest, _: i64) -> ResponseResult {
    let event = request.ok_or(response_error!("Empty track request"))?;
    let properties_by_name: HashMap<String, String> = request.properties.iter().collect();

    Ok(ResponseKind::Success)
}

pub fn handle_identify_request(request: TelemetryIdentifyRequest, _: i64) -> ResponseResult {}

impl TelemetryEvent {}
