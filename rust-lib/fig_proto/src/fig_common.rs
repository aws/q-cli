//! Fig.js Protocol Buffers

use serde_json::Value;

use crate::proto::fig_common::*;

impl From<String> for Json {
    fn from(s: String) -> Self {
        Self {
            value: Some(json::Value::String(s)),
        }
    }
}

impl From<u64> for Json {
    fn from(n: u64) -> Self {
        Self {
            value: Some(json::Value::Number(json::Number {
                number: Some(json::number::Number::U64(n)),
            })),
        }
    }
}

impl From<i64> for Json {
    fn from(n: i64) -> Self {
        Self {
            value: Some(json::Value::Number(json::Number {
                number: Some(json::number::Number::I64(n)),
            })),
        }
    }
}

impl From<f64> for Json {
    fn from(n: f64) -> Self {
        Self {
            value: Some(json::Value::Number(json::Number {
                number: Some(json::number::Number::F64(n)),
            })),
        }
    }
}

impl From<bool> for Json {
    fn from(b: bool) -> Self {
        Self {
            value: Some(json::Value::Bool(b)),
        }
    }
}

impl<T> From<Option<T>> for Json
where
    T: Into<Json>,
{
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(v) => v.into(),
            None => Self {
                value: Some(json::Value::Null(json::Null {})),
            },
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
                    number: n
                        .as_i64()
                        .map(json::number::Number::I64)
                        .or_else(|| n.as_u64().map(json::number::Number::U64))
                        .or_else(|| n.as_f64().map(json::number::Number::F64)),
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
            Some(json::Value::Number(n)) => match n.number {
                Some(json::number::Number::I64(i)) => i.into(),
                Some(json::number::Number::U64(u)) => u.into(),
                Some(json::number::Number::F64(f)) => f.into(),
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
