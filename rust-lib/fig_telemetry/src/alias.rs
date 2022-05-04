use std::collections::HashMap;

use crate::{
    Error,
    ALIAS_SUBDOMAIN,
    API_DOMAIN,
};

pub async fn emit_alias(user_id: String) -> Result<(), Error> {
    if fig_settings::settings::get_bool("telemetry.disabled")
        .ok()
        .flatten()
        .unwrap_or(false)
    {
        return Err(Error::TelemetryDisabled);
    }

    let alias = HashMap::from([("previousId", fig_auth::get_default("uuid")?), ("userId", user_id)]);

    // Emit it!
    reqwest::Client::new()
        .post(format!("{}{}", API_DOMAIN, ALIAS_SUBDOMAIN))
        .header("Content-Type", "application/json")
        .json(&alias)
        .send()
        .await?;

    Ok(())
}
