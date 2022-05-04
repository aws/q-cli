use anyhow::anyhow;
use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{
    GetSettingsPropertyRequest, GetSettingsPropertyResponse, UpdateSettingsPropertyRequest,
};
use fig_settings::settings;

use super::{RequestResult, RequestResultImpl};

pub async fn get(request: GetSettingsPropertyRequest) -> RequestResult {
    let value = match request.key {
        Some(key) => settings::get_value(&key)
            .map_err(|_| anyhow!("Failed getting settings value for {key}"))?
            .ok_or_else(|| anyhow!("No value for key"))?,
        None => settings::local_settings()
            .map(|s| s.inner)
            .map_err(|_| anyhow!("Failed getting settings"))?,
    };

    let json_blob = serde_json::to_string(&value)
        .map_err(|_| anyhow!("Could not convert value for key to JSON"))?;

    let response =
        ServerOriginatedSubMessage::GetSettingsPropertyResponse(GetSettingsPropertyResponse {
            json_blob: Some(json_blob),
            is_default: None,
        });

    Ok(response.into())
}

pub async fn update(request: UpdateSettingsPropertyRequest) -> RequestResult {
    match (&request.key, request.value) {
        (Some(key), Some(value)) => settings::set_value(key, value)
            .await
            .map_err(|_| anyhow!("Failed setting {key}"))?,
        (Some(key), None) => settings::remove_value(key)
            .await
            .map_err(|_| anyhow!("Failed removing {key}"))?,
        (None, _) => {
            return RequestResult::error("No key provided with request");
        }
    }

    RequestResult::success()
}
