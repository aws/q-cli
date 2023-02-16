use fig_util::system_info::get_system_id;
use serde_json::{
    Map,
    Value,
};

use crate::Error;

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
    if let Some(email) = fig_request::auth::get_email() {
        if let Some(domain) = email.split('@').last() {
            prop.insert("domain".into(), domain.into());
        }
        prop.insert("email".into(), email.into());
    }

    #[cfg(target_os = "macos")]
    prop.insert("desktop_version".into(), env!("CARGO_PKG_VERSION").into());

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

    #[cfg(target_os = "linux")]
    prop.insert("device_linux_wsl".into(), fig_util::system_info::in_wsl().into());

    #[cfg(unix)]
    prop.insert("device_ssh".into(), fig_util::system_info::in_ssh().into());

    #[cfg(target_os = "windows")]
    if let Some(fig_util::system_info::OSVersion::Windows { name, build }) = fig_util::system_info::os_version() {
        prop.insert("device_windows_name".into(), name.to_owned().into());
        prop.insert("device_windows_build".into(), build.to_owned().into());
    }

    #[cfg(target_os = "macos")]
    {
        let os_version = macos_utils::os::NSOperatingSystemVersion::get();
        prop.insert("device_macos_release_version".into(), os_version.to_string().into());
    }

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

    // Manifest data
    prop.insert("manifest_version".into(), env!("CARGO_PKG_VERSION").into());
    prop.insert(
        "manifest_variant".into(),
        fig_util::manifest::manifest()
            .as_ref()
            .map(|m| m.variant.to_string())
            .into(),
    );
    prop.insert(
        "manifest_kind".into(),
        fig_util::manifest::manifest()
            .as_ref()
            .map(|m| m.kind.to_string())
            .into(),
    );
    prop.insert(
        "manifest_managed_by".into(),
        fig_util::manifest::manifest()
            .as_ref()
            .map(|m| m.managed_by.to_string())
            .into(),
    );

    prop
}

pub(crate) async fn make_telemetry_request(route: &str, mut body: Map<String, Value>) -> Result<(), Error> {
    body.insert(
        "anonymousId".into(),
        fig_settings::state::get_or_create_anonymous_id()?.into(),
    );
    fig_request::Request::post(route)
        .maybe_auth()
        .body_json(body)
        .send()
        .await?;
    Ok(())
}
