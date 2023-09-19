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
    TrackSource,
    PAGE_SUBDOMAIN,
};

pub async fn emit_page<'a, I, K, V>(
    category: String,
    name: String,
    source: TrackSource,
    properties: I,
) -> Result<(), Error>
where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<Value>,
{
    if telemetry_is_disabled() {
        return Err(Error::TelemetryDisabled);
    }

    let mut props = crate::util::default_properties().await;
    props.insert("source".into(), source.to_string().into());
    props.extend(properties.into_iter().map(|(k, v)| (k.into(), v.into())));

    let mut body: Map<String, Value> = Map::new();
    body.insert("category".into(), category.into());
    body.insert("name".into(), name.into());
    body.insert("useUnprefixed".into(), true.into());
    body.insert("properties".into(), props.into());

    make_telemetry_request(PAGE_SUBDOMAIN, body).await
}
