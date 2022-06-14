use std::collections::HashMap;

use crate::util::{
    make_telemetry_request,
    telemetry_is_disabled,
};
use crate::{
    Error,
    ALIAS_SUBDOMAIN,
};

pub async fn emit_alias(user_id: String) -> Result<(), Error> {
    if telemetry_is_disabled() {
        return Err(Error::TelemetryDisabled);
    }

    let alias = HashMap::from([
        ("previousId".into(), fig_auth::get_default("anonymousId")?),
        ("userId".into(), user_id),
    ]);

    make_telemetry_request(ALIAS_SUBDOMAIN, alias).await
}
