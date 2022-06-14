use std::collections::HashMap;

use crate::util::{
    make_telemetry_request,
    telemetry_is_disabled,
};
use crate::{
    Error,
    IDENTIFY_SUBDOMAIN,
};

pub async fn emit_identify<'a, I, T>(traits: I) -> Result<(), Error>
where
    I: IntoIterator<Item = T>,
    T: Into<(&'a str, &'a str)>,
{
    if telemetry_is_disabled() {
        return Err(Error::TelemetryDisabled);
    }

    let mut identify = HashMap::new();

    for kv in traits.into_iter() {
        let (key, value) = kv.into();
        identify.insert(format!("prop_{key}"), value.into());
    }

    make_telemetry_request(IDENTIFY_SUBDOMAIN, identify).await
}
