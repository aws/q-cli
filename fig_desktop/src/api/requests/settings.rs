use anyhow::anyhow;
use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{
    GetSettingsPropertyRequest,
    GetSettingsPropertyResponse,
    UpdateSettingsPropertyRequest,
};
use fig_settings::settings;
use serde_json::Value;

use super::{
    RequestResult,
    RequestResultImpl,
};

pub async fn get(request: GetSettingsPropertyRequest) -> RequestResult {
    let value = match request.key {
        Some(key) => settings::get_value(&key)
            .map_err(|err| anyhow!("Failed getting settings value for {key}: {err}"))?
            .ok_or_else(|| anyhow!("No value for key '{key}'"))?,
        None => settings::local_settings()
            .map(|s| Value::Object(s.inner))
            .map_err(|_| anyhow!("Failed getting settings"))?,
    };

    let json_blob =
        serde_json::to_string(&value).map_err(|err| anyhow!("Could not convert value for key to JSON: {err}"))?;

    let response = ServerOriginatedSubMessage::GetSettingsPropertyResponse(GetSettingsPropertyResponse {
        json_blob: Some(json_blob),
        is_default: None,
    });

    Ok(response.into())
}

pub async fn update(request: UpdateSettingsPropertyRequest) -> RequestResult {
    match (&request.key, request.value) {
        (Some(key), Some(value)) => {
            let value = serde_json::from_str(&value).unwrap_or(serde_json::Value::String(value));
            fig_api_client::settings::update(key, value)
                .await
                .map_err(|err| anyhow!("Failed setting {key}: {err}"))?;
        },
        (Some(key), None) => fig_api_client::settings::delete(key)
            .await
            .map_err(|err| anyhow!("Failed removing {key}: {err}"))?,
        (None, _) => {
            return RequestResult::error("No key provided with request");
        },
    }

    RequestResult::success()
}
