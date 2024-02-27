use std::str::FromStr;

use uuid::{
    uuid,
    Uuid,
};

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
pub(crate) fn get_client_id() -> Uuid {
    if cfg!(test) {
        return uuid!("ffffffff-ffff-ffff-ffff-ffffffffffff");
    }

    if telemetry_is_disabled() {
        return uuid!("11111111-1111-1111-1111-111111111111");
    }

    match fig_settings::state::get_string("telemetryClientId")
        .ok()
        .flatten()
        .and_then(|s| Uuid::from_str(&s).ok())
    {
        Some(uuid) => uuid,
        None => {
            let uuid = Uuid::new_v4();
            let _ = fig_settings::state::set_value("telemetryClientId", uuid.to_string());
            uuid
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_telemetry_disabled() {
        let is_disabled = telemetry_is_disabled();
        assert!(!is_disabled);
    }

    #[test]
    fn test_get_client_id() {
        let id = get_client_id();
        assert!(!id.is_nil());
    }
}
