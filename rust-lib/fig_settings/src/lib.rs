pub mod keybindings;
pub mod remote_settings;
pub mod settings;
pub mod state;

use std::fs;
use std::path::PathBuf;

use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Url;
use serde_json::Value;
use thiserror::Error;

fn get_host_string(key: impl AsRef<str>) -> Option<Url> {
    state::get_value(key)
        .ok()
        .flatten()
        .and_then(|v| v.as_str().and_then(|s| Url::parse(s).ok()))
}

pub fn api_host() -> Url {
    get_host_string("developer.apiHost")
        .or_else(|| get_host_string("developer.cli.apiHost"))
        .unwrap_or_else(|| Url::parse("https://api.fig.io").unwrap())
}

static WS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\S+:|^)//").unwrap());

pub fn ws_host() -> Url {
    get_host_string("developer.wsHost")
        .or_else(|| get_host_string("developer.cli.wsHost"))
        .unwrap_or_else(|| {
            let host = api_host();
            Url::parse(&WS_REGEX.replace_all(host.as_str(), "wss://")).unwrap()
        })
}

pub type Map = serde_json::Map<String, Value>;

#[derive(Debug, Clone)]
pub struct LocalJson {
    pub inner: Map,
    pub path: PathBuf,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    RemoteSettingsError(#[from] remote_settings::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error(transparent)]
    FigUtilError(#[from] fig_util::Error),
    #[error("settings file is not a json object")]
    SettingsNotObject,
    #[error("could not get path to settings file")]
    SettingsPathNotFound,
}

impl LocalJson {
    pub fn load(path: impl Into<PathBuf>) -> Result<Self, Error> {
        let path = path.into();

        // If the folder doesn't exist, create it.
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        // If the file doesn't exist, create it.
        if !path.exists() {
            fs::File::create(&path)?;
        }

        let file = fs::read_to_string(&path)?;

        Ok(Self {
            inner: match serde_json::from_str(&file).or_else(|_| {
                if file.is_empty() {
                    Ok(serde_json::Value::Object(serde_json::Map::new()))
                } else {
                    Err(Error::SettingsNotObject)
                }
            })? {
                Value::Object(val) => val,
                _ => todo!(),
            },
            path,
        })
    }

    pub fn save(&self) -> Result<(), Error> {
        // If the folder doesn't exist, create it.
        if let Some(parent) = self.path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        // Write the file.
        fs::write(&self.path, serde_json::to_string_pretty(&self.inner)?)?;
        Ok(())
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) {
        self.inner.insert(key.into(), value.into());
    }

    pub fn get(&self, key: impl AsRef<str>) -> Option<&serde_json::Value> {
        self.inner.get(key.as_ref())
    }

    pub fn remove(&mut self, key: impl AsRef<str>) -> Option<Value> {
        self.inner.remove(key.as_ref())
    }

    pub fn get_mut(&mut self, key: impl Into<String>) -> Option<&mut serde_json::Value> {
        self.inner.get_mut(&key.into())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn local_json() {
        let dir = tempfile::tempdir().unwrap();
        let local_json_path = dir.path().join("local.json");

        let mut local_json = LocalJson::load(&local_json_path).unwrap();

        assert_eq!(fs::read_to_string(&local_json_path).unwrap(), "");
        assert_eq!(local_json.inner, serde_json::Map::new());

        local_json.save().unwrap();
        assert_eq!(fs::read_to_string(&local_json_path).unwrap(), "{}");

        local_json.set("a", 123);
        local_json.set("b", "hello");
        local_json.set("c", false);

        local_json.save().unwrap();
        assert_eq!(
            fs::read_to_string(&local_json_path).unwrap(),
            "{\n  \"a\": 123,\n  \"b\": \"hello\",\n  \"c\": false\n}"
        );

        local_json.remove("a").unwrap();

        local_json.save().unwrap();
        assert_eq!(
            fs::read_to_string(&local_json_path).unwrap(),
            "{\n  \"b\": \"hello\",\n  \"c\": false\n}"
        );

        assert_eq!(local_json.get("b").unwrap(), "hello");
    }

    #[test]
    fn local_json_errors() {
        let dir = tempfile::tempdir().unwrap();
        let local_json_path = dir.path().join("local.json");

        fs::write(&local_json_path, "hey").unwrap();
        assert!(matches!(
            LocalJson::load(&local_json_path).unwrap_err(),
            Error::SettingsNotObject
        ));
    }
}
