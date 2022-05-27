use std::env::{
    consts,
    var,
};

use camino::Utf8PathBuf;
use fnv::FnvBuildHasher;
use hashbrown::HashMap;
use once_cell::sync::Lazy;
use serde::{
    Deserialize,
    Serialize,
};

pub static CONSTANTS_SCRIPT: Lazy<String> = Lazy::new(|| Constants::default().script());

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constants {
    version: String,
    cli: Option<Utf8PathBuf>,
    bundle_path: Option<Utf8PathBuf>,
    remote: Option<String>,
    home: Option<Utf8PathBuf>,
    user: Option<String>,
    default_path: Option<String>,
    themes: Option<Vec<String>>,
    os: &'static str,
    arch: &'static str,
}

impl Default for Constants {
    fn default() -> Self {
        Self {
            version: Default::default(),
            cli: Default::default(),
            bundle_path: Default::default(),
            remote: Default::default(),
            home: fig_directories::home_dir().and_then(|dir| dir.try_into().ok()),
            user: Default::default(),
            default_path: var("PATH").ok(),
            themes: Default::default(),
            os: consts::OS,
            arch: consts::ARCH,
        }
    }
}

impl Constants {
    pub fn to_map(&self) -> HashMap<String, Option<String>, FnvBuildHasher> {
        let constants = self.clone();

        let mut h = HashMap::with_hasher(FnvBuildHasher::default());

        h.insert("version".into(), Some(constants.version));
        h.insert("cli".into(), constants.cli.map(String::from));
        h.insert("bundlePath".into(), constants.bundle_path.map(String::from));
        h.insert("remote".into(), constants.remote);
        h.insert("home".into(), constants.home.map(String::from));
        h.insert("user".into(), constants.user);
        h.insert("defaultPath".into(), constants.default_path);
        h.insert("themes".into(), constants.themes.map(|themes| themes.join("\n")));
        h.insert("os".into(), Some(constants.os.into()));
        h.insert("arch".into(), Some(constants.arch.into()));

        h
    }

    pub fn script(&self) -> String {
        self.to_map()
            .into_iter()
            .map(|(key, value)| {
                format!(
                    "fig.constants.{key} = {};",
                    value.map(|v| format!("'{v}'")).unwrap_or_else(|| "null".into())
                )
            })
            .collect::<Vec<String>>()
            .join("\n")
    }
}
