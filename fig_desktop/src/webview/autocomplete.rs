use tracing::error;
use url::Url;

pub const AUTOCOMPLETE_PRODUCTION_URL: &str = "https://autocomplete.fig.io";
pub const AUTOCOMPLETE_STAGING_URL: &str = "https://staging.autocomplete.fig.io";
pub const AUTOCOMPLETE_DEVELOP_URL: &str = "https://develop.autocomplete.fig.io";

pub fn url() -> Url {
    if let Some(dev_url) = fig_settings::settings::get_string_opt("developer.autocomplete.host") {
        match Url::parse(&dev_url) {
            Ok(url) => return url,
            Err(err) => {
                error!(%err, "Failed to parse developer.autocomplete.host");
            },
        }
    };

    match fig_settings::settings::get_string_opt("developer.autocomplete.build").as_deref() {
        Some("staging") => Url::parse(AUTOCOMPLETE_STAGING_URL).unwrap(),
        Some("develop") | Some("dev") => Url::parse(AUTOCOMPLETE_DEVELOP_URL).unwrap(),
        _ => Url::parse(AUTOCOMPLETE_PRODUCTION_URL).unwrap(),
    }
}
