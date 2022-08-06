use fig_settings::state;
use serde_json::{
    Map,
    Value,
};

use crate::Error;

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
    std::env::var_os("FIG_DISABLE_TELEMETRY").is_some()
        || fig_settings::settings::get_value("telemetry.disabled")
            .ok()
            .flatten()
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
}

pub(crate) fn default_properties() -> Map<String, Value> {
    let mut prop = Map::new();

    if let Some(email) = fig_auth::get_email() {
        if let Some(domain) = email.split('@').last() {
            prop.insert("prop_domain".into(), domain.into());
        }
        prop.insert("prop_email".into(), email.into());
    }

    #[cfg(target_os = "macos")]
    if let Ok(version) = fig_auth::get_default("versionAtPreviousLaunch") {
        if let Some((version, build)) = version.split_once(',') {
            prop.insert("prop_version".into(), version.into());
            prop.insert("prop_build".into(), build.into());
        }
    }

    prop.insert(
        "install_method".into(),
        crate::install_method::get_install_method().to_string().into(),
    );

    if let Ok(device_id) = fig_util::get_system_id() {
        prop.insert("device_id".into(), device_id.into());
    }

    prop.insert("device_os".into(), std::env::consts::OS.into());
    prop.insert("device_arch".into(), std::env::consts::ARCH.into());

    prop
}

pub(crate) async fn make_telemetry_request(route: &str, mut body: Map<String, Value>) -> Result<(), Error> {
    body.insert("anonymousId".into(), get_or_create_anonymous_id()?.into());
    fig_request::Request::post(route).auth().body(body).send().await?;
    Ok(())
}
