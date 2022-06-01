use std::env::{
    consts,
    var,
};

use camino::Utf8PathBuf;
use once_cell::sync::Lazy;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;

pub static CONSTANTS_SCRIPT: Lazy<String> = Lazy::new(|| Constants::default().script());

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constants {
    version: &'static str,
    cli: Option<Utf8PathBuf>,
    bundle_path: Option<Utf8PathBuf>,
    remote: Option<String>,
    home: Option<Utf8PathBuf>,
    user: String,
    default_path: Option<String>,
    themes: Option<Vec<String>>,
    os: &'static str,
    arch: &'static str,
}

impl Default for Constants {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION"),
            cli: Some("/usr/bin/fig".into()),
            bundle_path: None,
            remote: None,
            home: fig_directories::home_dir().and_then(|dir| dir.try_into().ok()),
            user: whoami::username(),
            default_path: var("PATH").ok(),
            themes: Some(vec!["light".into(), "dark".into()]),
            os: consts::OS,
            arch: consts::ARCH,
        }
    }
}

impl Constants {
    pub fn script(&self) -> String {
        format!("fig.constants = {}", json!(self))
    }
}
