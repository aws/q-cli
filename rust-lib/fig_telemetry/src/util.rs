pub fn telemetry_is_disabled() -> bool {
    fig_settings::settings::get_value("telemetry.disabled")
        .ok()
        .flatten()
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}
