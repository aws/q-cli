use std::str::FromStr;

use fig_os_shim::Env;
use fig_settings::Settings;
use uuid::{
    Uuid,
    uuid,
};

pub(crate) fn telemetry_is_disabled() -> bool {
    let is_test = cfg!(test);
    telemetry_is_disabled_inner(is_test, &Env::new(), &Settings::new())
}

/// Returns whether or not the user has disabled telemetry through settings or environment
fn telemetry_is_disabled_inner(is_test: bool, env: &Env, settings: &Settings) -> bool {
    let env_var = env.get_os("Q_DISABLE_TELEMETRY").is_some();
    let setting = !settings
        .get_value("telemetry.enabled")
        .ok()
        .flatten()
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    !is_test && (env_var || setting)
}

pub(crate) fn get_client_id() -> Uuid {
    get_client_id_inner(cfg!(test), &Env::new(), &Settings::new())
}

/// Generates or gets the client id and caches the result
///
/// Based on: <https://github.com/aws/aws-toolkit-vscode/blob/7c70b1909050043627e6a1471392e22358a15985/src/shared/telemetry/util.ts#L41C1-L62>
pub(crate) fn get_client_id_inner(is_test: bool, env: &Env, settings: &Settings) -> Uuid {
    if is_test {
        return uuid!("ffffffff-ffff-ffff-ffff-ffffffffffff");
    }

    if telemetry_is_disabled_inner(is_test, env, settings) {
        return uuid!("11111111-1111-1111-1111-111111111111");
    }

    if let Ok(client_id) = env.get("Q_TELEMETRY_CLIENT_ID") {
        if let Ok(uuid) = Uuid::from_str(&client_id) {
            return uuid;
        }
    }

    match settings
        .get_string("telemetryClientId")
        .ok()
        .flatten()
        .and_then(|s| Uuid::from_str(&s).ok())
    {
        Some(uuid) => uuid,
        None => {
            let uuid = Uuid::new_v4();
            let _ = settings.set_value("telemetryClientId", uuid.to_string());
            uuid
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_telemetry_disabled() {
        // disabled by default in tests
        // let is_disabled = telemetry_is_disabled();
        // assert!(!is_disabled);

        // let settings = Settings::new_fake();

        // let env = Env::from_slice(&[("Q_DISABLE_TELEMETRY", "1")]);
        // assert!(telemetry_is_disabled_inner(true, &env, &settings));
        // assert!(telemetry_is_disabled_inner(false, &env, &settings));

        // let env = Env::new_fake();
        // assert!(telemetry_is_disabled_inner(true, &env, &settings));
        // assert!(!telemetry_is_disabled_inner(false, &env, &settings));

        // settings.set_value("telemetry.enabled", false).unwrap();
        // assert!(telemetry_is_disabled_inner(false, &env, &settings));
        // assert!(!telemetry_is_disabled_inner(true, &env, &settings));

        // settings.set_value("telemetry.enabled", true).unwrap();
        // assert!(!telemetry_is_disabled_inner(false, &env, &settings));
        // assert!(!telemetry_is_disabled_inner(true, &env, &settings));
    }

    #[test]
    fn test_get_client_id() {
        // max by default in tests
        let id = get_client_id();
        assert!(id.is_max());

        let settings = Settings::new_fake();

        const TEST_UUID: &str = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";

        let env = Env::from_slice(&[("Q_TELEMETRY_CLIENT_ID", TEST_UUID)]);
        assert_eq!(get_client_id_inner(false, &env, &settings), uuid!(TEST_UUID));

        let env = Env::new_fake();
        assert!(get_client_id_inner(true, &env, &settings).is_max());

        settings.set_value("telemetryClientId", TEST_UUID).unwrap();
        assert_eq!(get_client_id_inner(false, &env, &settings), uuid!(TEST_UUID));

        settings.remove_value("telemetryClientId").unwrap();
        assert_eq!(
            get_client_id_inner(false, &env, &settings).to_string(),
            settings.get_string("telemetryClientId").unwrap().unwrap()
        );
    }
}
