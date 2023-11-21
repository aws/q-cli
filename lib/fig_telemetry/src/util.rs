use std::borrow::Cow;

/// Returns whether or not the user has disabled telemetry through settings or environment
pub fn telemetry_is_disabled() -> bool {
    let is_test = cfg!(test);
    let env_var = std::env::var_os("CW_DISABLE_TELEMETRY").is_some();
    let setting = !fig_settings::settings::get_value("telemetry.enabled")
        .ok()
        .flatten()
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    !is_test && (env_var || setting)
}

/// Generates or gets the client id and caches the result
///
/// Based on: <https://github.com/aws/aws-toolkit-vscode/blob/7c70b1909050043627e6a1471392e22358a15985/src/shared/telemetry/util.ts#L41C1-L62>
pub(crate) fn get_client_id() -> Cow<'static, str> {
    if cfg!(test) {
        return "ffffffff-ffff-ffff-ffff-ffffffffffff".into();
    }

    if telemetry_is_disabled() {
        return "11111111-1111-1111-1111-111111111111".into();
    }

    match fig_settings::state::get_string("telemetryClientId").ok().flatten() {
        Some(client_id) => client_id.into(),
        None => {
            let client_id = uuid::Uuid::new_v4().to_string();
            fig_settings::state::set_value("telemetryClientId", client_id.clone()).ok();
            client_id.into()
        },
    }
}
