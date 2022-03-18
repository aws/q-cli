pub fn api_host() -> String {
    fig_settings::state::get_value("developer.figcli.apiHost")
        .ok()
        .flatten()
        .and_then(|s| s.as_str().map(String::from))
        .unwrap_or_else(|| "https://api.fig.io".to_string())
}

pub fn ws_host() -> String {
    fig_settings::state::get_value("developer.figcli.wsHost")
        .ok()
        .flatten()
        .and_then(|s| s.as_str().map(String::from))
        .unwrap_or_else(|| "wss://api.fig.io".to_string())
}
