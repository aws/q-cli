use std::collections::HashMap;

use crate::{Error, API_DOMAIN, IDENTIFY_SUBDOMAIN};

pub async fn emit_identify(traits: &HashMap<String, String>) -> Result<(), Error> {
    if fig_settings::settings::get_bool("telemetry.disabled")
        .ok()
        .flatten()
        .unwrap_or(false)
    {
        return Err(Error::TelemetryDisabled);
    }

    let mut identify = HashMap::from([("userId".into(), fig_auth::get_default("uuid")?)]);

    for (key, value) in traits {
        identify.insert(format!("trait_{key}"), value.to_string());
    }

    // Emit it!
    reqwest::Client::new()
        .post(format!("{}{}", API_DOMAIN, IDENTIFY_SUBDOMAIN))
        .header("Content-Type", "application/json")
        .json(&identify)
        .send()
        .await?;

    Ok(())
}
