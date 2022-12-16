use tracing::error;
use url::Url;

pub const DASHBOARD_PRODUCTION_URL: &str = "https://desktop.fig.io";
pub const DASHBOARD_STAGING_URL: &str = "https://staging.desktop.fig.io";
pub const DASHBOARD_DEVELOP_URL: &str = "https://develop.desktop.fig.io";

pub fn url() -> Url {
    if let Some(dev_url) = fig_settings::settings::get_string_opt("developer.dashboard.host") {
        match Url::parse(&dev_url) {
            Ok(url) => return url,
            Err(err) => {
                error!(%err, "Failed to parse developer.dashboard.host");
            },
        }
    };

    match fig_settings::settings::get_string_opt("developer.dashboard.build").as_deref() {
        Some("staging" | "beta") => Url::parse(DASHBOARD_STAGING_URL).unwrap(),
        Some("develop" | "dev") => Url::parse(DASHBOARD_DEVELOP_URL).unwrap(),
        _ => Url::parse(DASHBOARD_PRODUCTION_URL).unwrap(),
    }
}
