use anyhow::anyhow;
use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{
    GetLocalStateRequest,
    GetLocalStateResponse,
    UpdateLocalStateRequest,
};
use fig_settings::state;

use super::{
    RequestResult,
    RequestResultImpl,
};

pub async fn get(request: GetLocalStateRequest) -> RequestResult {
    let value = match request.key {
        Some(key) => state::get_value(&key)
            .map_err(|_| anyhow!("Failed getting settings value for {key}"))?
            .ok_or_else(|| anyhow!("No value for key"))?,
        None => state::local_settings()
            .map(|s| s.inner)
            .map_err(|_| anyhow!("Failed getting settings"))?,
    };

    let json_blob = serde_json::to_string(&value).map_err(|_| anyhow!("Could not convert value for key to JSON"))?;

    let response = ServerOriginatedSubMessage::GetLocalStateResponse(GetLocalStateResponse {
        json_blob: Some(json_blob),
    });

    Ok(response.into())
}

pub async fn update(request: UpdateLocalStateRequest) -> RequestResult {
    match (&request.key, request.value) {
        (Some(key), Some(value)) => state::set_value(key, value).map_err(|_| anyhow!("Failed setting {key}"))?,
        (Some(key), None) => state::remove_value(key).map_err(|_| anyhow!("Failed removing {key}"))?,
        (None, _) => {
            return RequestResult::error("No key provided with request");
        },
    }

    RequestResult::success()
}
