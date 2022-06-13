use serde::Serialize;

use crate::{
    Error,
    API_DOMAIN,
};

pub fn telemetry_is_disabled() -> bool {
    fig_settings::settings::get_value("telemetry.disabled")
        .ok()
        .flatten()
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

pub(crate) async fn make_telemetry_request<T>(route: &str, body: &T) -> Result<(), Error>
where
    T: Serialize + ?Sized,
{
    // Emit it!
    let mut request = reqwest::Client::new().post(format!("{}{}", API_DOMAIN, route));

    if let Ok(token) = fig_auth::get_token().await {
        request = request.bearer_auth(token);
    }

    request
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    Ok(())
}
