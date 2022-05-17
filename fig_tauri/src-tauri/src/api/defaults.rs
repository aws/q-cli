use std::path::PathBuf;

use anyhow::{
    anyhow,
    Result,
};
use fig_proto::fig::defaults_value::Type;
use fig_proto::fig::server_originated_message::Submessage as ServerOriginatedSubMessage;
use fig_proto::fig::{
    DefaultsValue,
    GetDefaultsPropertyRequest,
    GetDefaultsPropertyResponse,
    UpdateDefaultsPropertyRequest,
};
use serde_json::Value;
use tokio::fs;

use super::{
    RequestResult,
    RequestResultImpl,
};

fn path() -> Result<PathBuf> {
    Ok(fig_directories::fig_data_dir()
        .ok_or_else(|| anyhow!("Failed to get data dir"))?
        .join("defaults.json"))
}

pub async fn get(request: GetDefaultsPropertyRequest) -> RequestResult {
    let value = match request.key {
        Some(ref key) => fs::read(&path()?).await.ok().and_then(|file| {
            let mut value: Value = serde_json::from_slice(&file).ok()?;
            match value.get_mut(key).map(|v| v.take()).unwrap_or(Value::Null) {
                Value::Null => Some(Type::Null(true)),
                Value::Bool(b) => Some(Type::Boolean(b)),
                Value::Number(i) => i.as_i64().map(|i| Type::Integer(i)),
                Value::String(s) => Some(Type::String(s)),
                _ => None,
            }
        }),
        None => return Err(anyhow!("No key provided")),
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
            let path = path()?;
            if !path.exists() {
                match path.parent() {
                    Some(parent) if !parent.exists() => fs::create_dir_all(parent).await?,
                    _ => {},
                }
                fs::write(&path, b"{}").await?;
            }
            let file = fs::read(&path).await?;
            let mut object: Value = serde_json::from_slice(&file)?;
            if let Some(object) = object.as_object_mut() {
                object.remove(&key);
            }
            fs::write(&path, serde_json::to_vec(&object)?).await?;

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
                _ => unreachable!(),
            };

            let path = path()?;
            if !path.exists() {
                match path.parent() {
                    Some(parent) if !parent.exists() => fs::create_dir_all(parent).await?,
                    _ => {},
                }
                fs::write(&path, b"{}").await?;
            }
            let file = fs::read(&path).await?;
            let mut object: Value = serde_json::from_slice(&file)?;
            if let Some(object) = object.as_object_mut() {
                object.insert(key, value);
            }
            fs::write(&path, serde_json::to_vec(&object)?).await?;

            RequestResultImpl::success()
        },
        (Some(_), Some(_)) => Err(anyhow!("Value is an unsupported type")),
        (None, _) => Err(anyhow!("No key provider")),
    }
}
