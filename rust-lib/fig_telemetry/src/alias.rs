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

    let alias = [
        #[cfg(target_os = "macos")]
        ("previousId".into(), fig_auth::get_default("anonymousId")?.into()),
        ("userId".into(), user_id.into()),
    ]
    .into_iter()
    .collect();

    make_telemetry_request(ALIAS_SUBDOMAIN, alias).await
}
