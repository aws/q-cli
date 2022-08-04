use std::env::{
    consts,
    var,
};

use camino::Utf8PathBuf;
use fig_util::directories;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use which::which;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Constants {
    version: &'static str,
    cli: Option<Utf8PathBuf>,
    bundle_path: Option<Utf8PathBuf>,
    remote: Option<String>,
    home: Option<Utf8PathBuf>,
    user: String,
    default_path: Option<String>,
    themes_folder: Option<Utf8PathBuf>,
    themes: Option<Vec<String>>,
    os: &'static str,
    arch: &'static str,
}

impl Default for Constants {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION"),
            cli: which("fig").ok().and_then(|exe| Utf8PathBuf::try_from(exe).ok()),
            bundle_path: None,
            remote: None,
            home: directories::home_dir().ok().and_then(|dir| dir.try_into().ok()),
            user: whoami::username(),
            default_path: var("PATH").ok(),
            themes_folder: fig_install::themes::themes_directory()
                .ok()
                .and_then(|dir| Utf8PathBuf::try_from(dir).ok()),
            themes: Some(vec!["light".into(), "dark".into(), "system".into()]),
            os: consts::OS,
            arch: consts::ARCH,
        }
    }
}

impl Constants {
    pub fn script(&self) -> String {
        format!("fig.constants = {};", json!(self))
    }
}
