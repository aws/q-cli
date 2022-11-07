use url::Url;

use crate::state;

pub fn host() -> Url {
    std::env::var("FIG_API_HOST")
        .ok()
        .map(|host| Url::parse(&host).unwrap())
        .or_else(|| get_host_string("developer.apiHost"))
        .or_else(|| get_host_string("developer.cli.apiHost"))
        .unwrap_or_else(|| Url::parse("https://api.fig.io").unwrap())
}

pub fn release_host() -> Url {
    std::env::var("FIG_RELEASE_API_HOST")
        .ok()
        .map(|host| Url::parse(&host).unwrap())
        .or_else(|| get_host_string("developer.release.apiHost"))
        .unwrap_or_else(|| Url::parse("https://release.fig.io").unwrap())
}

pub fn ws_host() -> Url {
    std::env::var("FIG_WS_HOST")
        .ok()
        .map(|host| Url::parse(&host).unwrap())
        .or_else(|| get_host_string("developer.wsHost"))
        .or_else(|| get_host_string("developer.cli.wsHost"))
        .unwrap_or_else(|| Url::parse("wss://ws.fig.io").unwrap())
}

fn get_host_string(key: impl AsRef<str>) -> Option<Url> {
    state::get_value(key)
        .ok()
        .flatten()
        .and_then(|v| v.as_str().and_then(|s| Url::parse(s).ok()))
}
