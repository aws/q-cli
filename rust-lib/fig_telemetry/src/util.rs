use std::collections::HashMap;

use fig_settings::state;

use crate::{
    Error,
    API_DOMAIN,
};

fn create_anonymous_id() -> anyhow::Result<String> {
    let anonymous_id = uuid::Uuid::new_v4().as_hyphenated().to_string();
    state::set_value("anonymousId", anonymous_id.clone())?;
    Ok(anonymous_id)
}

pub fn get_or_create_anonymous_id() -> anyhow::Result<String> {
    if let Ok(Some(anonymous_id)) = state::get_string("anonymousId") {
        return Ok(anonymous_id);
    }

    create_anonymous_id()
}

pub fn telemetry_is_disabled() -> bool {
    fig_settings::settings::get_value("telemetry.disabled")
        .ok()
        .flatten()
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

pub(crate) async fn make_telemetry_request(route: &str, mut body: HashMap<String, String>) -> Result<(), Error> {
    // Emit it!
    let mut request = reqwest::Client::new().post(format!("{}{}", API_DOMAIN, route));

    if let Ok(token) = fig_auth::get_token().await {
        request = request.bearer_auth(token);
    }

    body.insert("anonymousId".into(), get_or_create_anonymous_id()?);

    request
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    Ok(())
}
