use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{
    GetSettingsPropertyRequest, GetSettingsPropertyResponse, UpdateSettingsPropertyRequest,
};

use fig_settings::settings;

use crate::api::ResponseKind;
use crate::response_error;

use super::ResponseResult;

pub async fn get(request: GetSettingsPropertyRequest, _message_id: i64) -> ResponseResult {
    let value = match request.key {
        Some(key) => settings::get_value(key)
            .map_err(response_error!("Failed getting settings value"))?
            .ok_or_else(|| ResponseKind::Error(String::from("No value for key")))?,
        None => settings::local_settings()
            .map(|s| s.inner)
            .map_err(response_error!("Failed getting settings"))?,
    };

    let json_blob = serde_json::to_string(&value)
        .map_err(response_error!("Could not convert value for key to JSON"))?;

    let response =
        ServerOriginatedSubMessage::GetSettingsPropertyResponse(GetSettingsPropertyResponse {
            json_blob: Some(json_blob),
            is_default: None,
        });

    Ok(response.into())
}

pub async fn update(request: UpdateSettingsPropertyRequest, _message_id: i64) -> ResponseResult {
    match (request.key, request.value) {
        (Some(key), Some(value)) => {
            settings::set_value(key, value)
                .await
                .map_err(response_error!("Failed setting settings value"))?
                .map_err(response_error!("Failed setting settings value"))?;
        }
        (Some(key), None) => {
            settings::remove_value(key)
                .await
                .map_err(response_error!("Failed removing settings value"))?
                .map_err(response_error!("Failed removing settings value"))?;
        }
        (None, _) => {
            return Err(ResponseKind::Error(String::from(
                "No key provided with request",
            )));
        }
    }

    Ok(ResponseKind::Success)
}
