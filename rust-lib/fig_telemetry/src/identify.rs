use std::fmt::Display;

use serde_json::Value;

use crate::util::{
    make_telemetry_request,
    telemetry_is_disabled,
};
use crate::{
    Error,
    IDENTIFY_SUBDOMAIN,
};

pub async fn emit_identify<'a, I, K, V>(traits: I) -> Result<(), Error>
where
    I: IntoIterator<Item = (K, V)>,
    K: Display,
    V: Into<Value>,
{
    if telemetry_is_disabled() {
        return Err(Error::TelemetryDisabled);
    }

    make_telemetry_request(
        IDENTIFY_SUBDOMAIN,
        traits
            .into_iter()
            .map(|(key, value)| (format!("prop_{key}"), value.into()))
            .collect(),
    )
    .await
}
