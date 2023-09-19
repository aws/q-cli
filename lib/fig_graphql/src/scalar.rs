use serde::{
    Deserialize,
    Serialize,
};
pub use serde_json::Value as Json;
pub type JsonObject = serde_json::Map<String, Json>;

/// A light wrapper around [time::OffsetDateTime] to implement [time::serde::iso8601]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DateTime(#[serde(with = "time::serde::iso8601")] time::OffsetDateTime);

impl From<time::OffsetDateTime> for DateTime {
    fn from(dt: time::OffsetDateTime) -> Self {
        Self(dt)
    }
}

impl From<DateTime> for time::OffsetDateTime {
    fn from(dt: DateTime) -> Self {
        dt.0
    }
}

/// Arbitrary precision is enabled for `serde_json` so this will work
pub type BigInt = serde_json::Number;

pub type UnsignedInt = u64;
