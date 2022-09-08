use fig_settings::state;
use fig_util::system_info::get_system_id;
use serde_json::{
    Map,
    Value,
};

use crate::Error;

fn create_anonymous_id() -> Result<String, fig_settings::Error> {
    let anonymous_id = uuid::Uuid::new_v4().as_hyphenated().to_string();
    state::set_value("anonymousId", anonymous_id.clone())?;
    Ok(anonymous_id)
}

pub fn get_or_create_anonymous_id() -> Result<String, fig_settings::Error> {
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

    // legacy, to remove
    if let Some(email) = fig_auth::get_email() {
        if let Some(domain) = email.split('@').last() {
            prop.insert("domain".into(), domain.into());
        }
        prop.insert("email".into(), email.into());
    }

    #[cfg(target_os = "macos")]
    if let Ok(version) = fig_auth::get_default("versionAtPreviousLaunch") {
        if let Some((version, build)) = version.split_once(',') {
            prop.insert("desktop_version".into(), version.into());
            prop.insert("desktop_legacy_build".into(), build.into());
        }
    }

    #[cfg(target_os = "linux")]
    if let Some(linux_os_release) = fig_util::system_info::linux::get_os_release() {
        prop.insert("device_linux_release_id".into(), linux_os_release.id.as_deref().into());
        prop.insert(
            "device_linux_release_name".into(),
            linux_os_release.name.as_deref().into(),
        );
        prop.insert(
            "device_linux_release_version".into(),
            linux_os_release.version.as_deref().into(),
        );
        prop.insert(
            "device_linux_release_version_id".into(),
            linux_os_release.version_id.as_deref().into(),
        );
        prop.insert(
            "device_linux_release_variant".into(),
            linux_os_release.variant.as_deref().into(),
        );
        prop.insert(
            "device_linux_release_variant_id".into(),
            linux_os_release.variant_id.as_deref().into(),
        );
        prop.insert(
            "device_linux_release_build_id".into(),
            linux_os_release.build_id.as_deref().into(),
        );
    }

    #[cfg(target_os = "linux")]
    if let Ok(desktop) = std::env::var("XDG_SESSION_DESKTOP") {
        prop.insert("device_linux_environment_desktop".into(), desktop.into());
    } else if let Ok(desktop) = std::env::var("DESKTOP_SESSION") {
        prop.insert("device_linux_environment_desktop".into(), desktop.into());
    }

    #[cfg(target_os = "linux")]
    prop.insert(
        "device_linux_environment_display_server".into(),
        match std::env::var("XDG_SESSION_TYPE") {
            Ok(desktop) => desktop.into(),
            Err(_) => "x11".into(),
        },
    );

    #[cfg(target_os = "windows")]
    if let Some(fig_util::system_info::OSVersion::Windows { name, build }) = fig_util::system_info::os_version() {
        prop.insert("device_windows_name".into(), name.to_owned().into());
        prop.insert("device_windows_build".into(), build.to_owned().into());
    }

    // TODO(matt): add macos release

    prop.insert(
        "device_install_method".into(),
        crate::install_method::get_install_method().to_string().into(),
    );

    if let Ok(device_id) = get_system_id() {
        prop.insert("device_id".into(), device_id.into());
    }

    prop.insert("device_desktop".into(), true.into());
    prop.insert("device_os".into(), std::env::consts::OS.into());
    prop.insert("device_arch".into(), std::env::consts::ARCH.into());

    if let Some(manifest_version) = fig_util::manifest::version() {
        prop.insert("manifest_version".into(), manifest_version.into());
    }

    prop
}

pub(crate) async fn make_telemetry_request(route: &str, mut body: Map<String, Value>) -> Result<(), Error> {
    body.insert("anonymousId".into(), get_or_create_anonymous_id()?.into());
    fig_request::Request::post(route).auth().body(body).send().await?;
    Ok(())
}
