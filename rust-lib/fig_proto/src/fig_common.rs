//! Fig.js Protocal Buffers

use serde_json::Value;

use crate::proto::fig_common::*;

impl From<String> for Json {
    fn from(s: String) -> Self {
        Self {
            value: Some(json::Value::String(s)),
        }
    }
}

impl<K, V> FromIterator<(K, V)> for Json
where
    K: Into<String>,
    V: Into<Json>,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Json {
            value: Some(json::Value::Object(json::Object {
                map: iter.into_iter().map(|(k, v)| (k.into(), v.into())).collect(),
            })),
        }
    }
}

impl<I> FromIterator<I> for Json
where
    I: Into<Json>,
{
    fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
        Json {
            value: Some(json::Value::Array(json::Array {
                array: iter.into_iter().map(|i| i.into()).collect(),
            })),
        }
    }
}

impl From<Value> for Json {
    fn from(value: Value) -> Self {
        Self {
            value: Some(match value {
                Value::Null => json::Value::Null(json::Null {}),
                Value::Bool(b) => json::Value::Bool(b),
                Value::Number(n) => json::Value::Number(json::Number {
                    int: n
                        .as_i64()
                        .map(json::number::Int::I64)
                        .or_else(|| n.as_u64().map(json::number::Int::U64))
                        .or_else(|| n.as_f64().map(json::number::Int::F64)),
                }),
                Value::String(s) => json::Value::String(s),
                Value::Array(a) => json::Value::Array(json::Array {
                    array: a.into_iter().map(Json::from).collect(),
                }),
                Value::Object(o) => json::Value::Object(json::Object {
                    map: o.into_iter().map(|(k, v)| (k, Json::from(v))).collect(),
                }),
            }),
        }
    }
}

impl From<Json> for Value {
    fn from(json: Json) -> Self {
        match json.value {
            Some(json::Value::Null(_)) => Value::Null,
            Some(json::Value::Bool(b)) => b.into(),
            Some(json::Value::Number(n)) => match n.int {
                Some(json::number::Int::I64(i)) => i.into(),
                Some(json::number::Int::U64(u)) => u.into(),
                Some(json::number::Int::F64(f)) => f.into(),
                None => Value::Null,
            },
            Some(json::Value::String(s)) => s.into(),
            Some(json::Value::Array(a)) => Value::Array(a.array.into_iter().map(Value::from).collect()),
            Some(json::Value::Object(o)) => Value::Object(
                o.map
                    .into_iter()
                    .map(|(key, value)| (key, Value::from(value)))
                    .collect(),
            ),
            None => todo!(),
        }
    }
}
