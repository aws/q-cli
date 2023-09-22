use tracing::error;
use url::Url;

pub const AUTOCOMPLETE_PRODUCTION_URL: &str = "https://autocomplete.fig.io";
pub const AUTOCOMPLETE_STAGING_URL: &str = "https://staging.autocomplete.fig.io";
pub const AUTOCOMPLETE_DEVELOP_URL: &str = "https://develop.autocomplete.fig.io";

pub const AUTOCOMPLETE_PRODUCTION_FIGAPP_URL: &str = "figapp://autocomplete.localhost";
pub const AUTOCOMPLETE_STAGING_FIGAPP_URL: &str = "figapp://staging.autocomplete.localhost";
pub const AUTOCOMPLETE_DEVELOP_FIGAPP_URL: &str = "figapp://develop.autocomplete.localhost";

pub fn url() -> Url {
    if let Ok(autocomplete_url) = std::env::var("AUTOCOMPLETE_URL") {
        return Url::parse(&autocomplete_url).unwrap();
    }

    if let Some(dev_url) = fig_settings::settings::get_string_opt("developer.autocomplete.host") {
        match Url::parse(&dev_url) {
            Ok(url) => return dbg!(url),
            Err(err) => {
                error!(%err, "Failed to parse developer.autocomplete.host");
            },
        }
    };

    let offline_mode = fig_settings::settings::get_bool_or("autocomplete.offline-support", true);

    match (
        fig_settings::settings::get_string_opt("developer.autocomplete.build").as_deref(),
        offline_mode,
    ) {
        (Some("staging" | "beta"), false) => Url::parse(AUTOCOMPLETE_STAGING_URL).unwrap(),
        (Some("develop" | "dev"), false) => Url::parse(AUTOCOMPLETE_DEVELOP_URL).unwrap(),
        (_, false) => Url::parse(AUTOCOMPLETE_PRODUCTION_URL).unwrap(),
        (Some("staging" | "beta"), true) => Url::parse(AUTOCOMPLETE_STAGING_FIGAPP_URL).unwrap(),
        (Some("develop" | "dev"), true) => Url::parse(AUTOCOMPLETE_DEVELOP_FIGAPP_URL).unwrap(),
        (_, true) => Url::parse(AUTOCOMPLETE_PRODUCTION_FIGAPP_URL).unwrap(),
    }
}
