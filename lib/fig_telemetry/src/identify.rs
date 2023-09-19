use serde_json::{
    Map,
    Value,
};

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
    K: Into<String>,
    V: Into<Value>,
{
    if telemetry_is_disabled() {
        return Err(Error::TelemetryDisabled);
    }

    let mut body = Map::new();
    body.insert(
        "traits".into(),
        traits.into_iter().map(|(k, v)| (k.into(), v.into())).collect(),
    );
    body.insert("useUnprefixed".into(), true.into());

    make_telemetry_request(IDENTIFY_SUBDOMAIN, body).await
}
