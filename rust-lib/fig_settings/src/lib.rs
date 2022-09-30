pub mod keybindings;
pub mod settings;
pub mod state;

use std::fs::{
    self,
    File,
};
use std::io::{
    Read,
    Write,
};
use std::path::PathBuf;

use fd_lock::RwLock as FileRwLock;
use fig_util::directories;
use parking_lot::RwLock;
use serde_json::Value;
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error(transparent)]
    FigUtilError(#[from] fig_util::Error),
    #[error("settings file is not a json object")]
    SettingsNotObject,
    #[error(transparent)]
    DirectoryError(#[from] directories::DirectoryError),
}

pub type Map = serde_json::Map<String, Value>;

static STATE_LOCK: RwLock<()> = RwLock::new(());
static SETTINGS_LOCK: RwLock<()> = RwLock::new(());

#[derive(Debug, Clone, Copy)]
pub enum JsonType {
    State,
    Settings,
}

impl JsonType {
    pub fn path(&self) -> Result<PathBuf, Error> {
        match self {
            JsonType::State => Ok(directories::state_path()?),
            JsonType::Settings => Ok(directories::settings_path()?),
        }
    }

    fn lock(&self) -> &'static RwLock<()> {
        match self {
            JsonType::State => &STATE_LOCK,
            JsonType::Settings => &SETTINGS_LOCK,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LocalJson {
    pub inner: Map,
    pub json_type: JsonType,
}

impl LocalJson {
    pub fn load(json_type: JsonType) -> Result<Self, Error> {
        let path = json_type.path()?;

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

        let (string, res) = {
            let _lock_guard = json_type.lock().read();
            let mut file = FileRwLock::new(File::open(&path)?);
            let mut read = file.write()?;
            let mut string = String::new();
            let res = read.read_to_string(&mut string);
            (string, res)
        };

        res?;

        Ok(Self {
            inner: match serde_json::from_str(&string).or_else(|_| {
                if string.is_empty() {
                    Ok(serde_json::Value::Object(serde_json::Map::new()))
                } else {
                    Err(Error::SettingsNotObject)
                }
            })? {
                Value::Object(val) => val,
                _ => unreachable!(),
            },
            json_type,
        })
    }

    pub fn save(&self) -> Result<(), Error> {
        let path = self.json_type.path()?;
        // If the folder doesn't exist, create it.
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let json = serde_json::to_vec_pretty(&self.inner)?;

        let res = {
            let _lock_guard = self.json_type.lock().write();
            let mut file = FileRwLock::new(File::create(&path)?);
            let mut lock = file.write()?;
            lock.write_all(&json)
        };
        res?;

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

pub fn api_host() -> Url {
    get_host_string("developer.apiHost")
        .or_else(|| get_host_string("developer.cli.apiHost"))
        .unwrap_or_else(|| Url::parse("https://api.fig.io").unwrap())
}

pub fn ws_host() -> Url {
    get_host_string("developer.wsHost")
        .or_else(|| get_host_string("developer.cli.wsHost"))
        .unwrap_or_else(|| {
            let mut host = api_host();
            host.set_scheme(match host.scheme() {
                "http" => "ws",
                "https" => "wss",
                _ => "wss",
            })
            .unwrap();
            host
        })
}

fn get_host_string(key: impl AsRef<str>) -> Option<Url> {
    state::get_value(key)
        .ok()
        .flatten()
        .and_then(|v| v.as_str().and_then(|s| Url::parse(s).ok()))
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use super::*;

    fn test_store_type(path: &Path, store: JsonType) {
        let mut local_json = LocalJson::load(store).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "");
        assert_eq!(local_json.inner, serde_json::Map::new());
        local_json.save().unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "{}");

        local_json.set("a", 123);
        local_json.set("b", "hello");
        local_json.set("c", false);
        local_json.save().unwrap();
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "{\n  \"a\": 123,\n  \"b\": \"hello\",\n  \"c\": false\n}"
        );

        local_json.remove("a").unwrap();
        local_json.save().unwrap();
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "{\n  \"b\": \"hello\",\n  \"c\": false\n}"
        );
        assert_eq!(local_json.get("b").unwrap(), "hello");

        fs::write(&path, "invalid json").unwrap();
        assert!(matches!(LocalJson::load(store).unwrap_err(), Error::SettingsNotObject));
    }

    #[fig_test::test]
    fn test_settings_raw() {
        let path = tempfile::tempdir().unwrap().into_path().join("local.json");
        std::env::set_var("FIG_DIRECTORIES_SETTINGS_PATH", &path);
        test_store_type(&path, JsonType::Settings);
    }

    #[fig_test::test]
    fn test_state_raw() {
        let path = tempfile::tempdir().unwrap().into_path().join("local.json");
        std::env::set_var("FIG_DIRECTORIES_STATE_PATH", &path);
        test_store_type(&path, JsonType::State);
    }
}
