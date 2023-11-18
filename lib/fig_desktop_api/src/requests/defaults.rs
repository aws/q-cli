use fig_proto::fig::defaults_value::Type;
use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{
    DefaultsValue,
    GetDefaultsPropertyRequest,
    GetDefaultsPropertyResponse,
    UpdateDefaultsPropertyRequest,
};
use fig_util::directories;
use serde_json::Value;
use tokio::fs;

use super::{
    RequestResult,
    RequestResultImpl,
};

pub async fn get(request: GetDefaultsPropertyRequest) -> RequestResult {
    let value = match request.key {
        Some(ref key) => fs::read(
            &directories::fig_data_dir()
                .map_err(|err| err.to_string())?
                .join("defaults.json"),
        )
        .await
        .ok()
        .and_then(|file| {
            let mut value: Value = serde_json::from_slice(&file).ok()?;
            match value.get_mut(key).map_or(Value::Null, |v| v.take()) {
                Value::Null => Some(Type::Null(true)),
                Value::Bool(b) => Some(Type::Boolean(b)),
                Value::Number(i) => i.as_i64().map(Type::Integer),
                Value::String(s) => Some(Type::String(s)),
                _ => None,
            }
        }),
        None => return Err("No key provided".into()),
    };

    let response = ServerOriginatedSubMessage::GetDefaultsPropertyResponse(GetDefaultsPropertyResponse {
        key: request.key,
        value: Some(DefaultsValue { r#type: value }),
    });

    Ok(response.into())
}

pub async fn update(request: UpdateDefaultsPropertyRequest) -> RequestResult {
    match (request.key, request.value) {
        (
            Some(key),
            Some(DefaultsValue {
                r#type: Some(Type::Null(true)),
            })
            | None,
        ) => {
            let path = directories::fig_data_dir()
                .map_err(|err| err.to_string())?
                .join("defaults.json");
            if !path.exists() {
                match path.parent() {
                    Some(parent) if !parent.exists() => {
                        fs::create_dir_all(parent).await.map_err(|err| err.to_string())?;
                    },
                    _ => {},
                }
                fs::write(&path, b"{}").await.map_err(|err| err.to_string())?;
            }
            let file = fs::read(&path).await.map_err(|err| err.to_string())?;
            let mut object: Value = serde_json::from_slice(&file).map_err(|err| err.to_string())?;
            if let Some(object) = object.as_object_mut() {
                object.remove(&key);
            }
            fs::write(&path, serde_json::to_vec(&object).map_err(|err| err.to_string())?)
                .await
                .map_err(|err| err.to_string())?;

            RequestResultImpl::success()
        },
        (
            Some(key),
            Some(DefaultsValue {
                r#type: Some(t @ (Type::Boolean(_) | Type::String(_) | Type::Integer(_))),
            }),
        ) => {
            let value = match t {
                Type::String(s) => Value::from(s),
                Type::Boolean(b) => Value::from(b),
                Type::Integer(i) => Value::from(i),
                Type::Null(_) => unreachable!(),
            };

            let path = directories::fig_data_dir()
                .map_err(|err| err.to_string())?
                .join("defaults.json");
            if !path.exists() {
                match path.parent() {
                    Some(parent) if !parent.exists() => {
                        fs::create_dir_all(parent).await.map_err(|err| err.to_string())?;
                    },
                    _ => {},
                }
                fs::write(&path, "{}").await.map_err(|err| err.to_string())?;
            }
            let file = fs::read(&path).await.map_err(|err| err.to_string())?;
            let mut object: Value = serde_json::from_slice(&file).map_err(|err| err.to_string())?;
            if let Some(object) = object.as_object_mut() {
                object.insert(key, value);
            }
            fs::write(&path, serde_json::to_vec(&object).map_err(|err| err.to_string())?)
                .await
                .map_err(|err| err.to_string())?;

            RequestResultImpl::success()
        },
        (Some(_), Some(_)) => Err("Value is an unsupported type".into()),
        (None, _) => Err("No key provider".into()),
    }
}
