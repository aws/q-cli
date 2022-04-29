use std::collections::HashMap;

use crate::{Error, API_DOMAIN, IDENTIFY_SUBDOMAIN};

pub async fn emit_identify<'a, I, T>(traits: I) -> Result<(), Error>
where
    I: IntoIterator<Item = T>,
    T: Into<(&'a str, &'a str)>,
{
    if fig_settings::settings::get_bool("telemetry.disabled")
        .ok()
        .flatten()
        .unwrap_or(false)
    {
        return Err(Error::TelemetryDisabled);
    }

    let mut identify = HashMap::from([("userId".into(), fig_auth::get_default("uuid")?)]);

    for kv in traits.into_iter() {
        let (key, value) = kv.into();
        identify.insert(format!("prop_{key}"), value.into());
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
