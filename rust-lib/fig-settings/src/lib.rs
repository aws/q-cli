pub mod remote_settings;
pub mod settings;
pub mod state;

use std::{fs, path::PathBuf};

use thiserror::Error;

pub fn api_host() -> String {
    state::get_value("developer.figcli.apiHost")
        .ok()
        .flatten()
        .and_then(|s| s.as_str().map(String::from))
        .unwrap_or_else(|| "https://api.fig.io".to_string())
}

pub fn ws_host() -> String {
    state::get_value("developer.figcli.wsHost")
        .ok()
        .flatten()
        .and_then(|s| s.as_str().map(String::from))
        .unwrap_or_else(|| "wss://api.fig.io".to_string())
}

pub struct LocalJson {
    inner: serde_json::Value,
    path: PathBuf,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Remote settings error: {0}")]
    RemoteSettingsError(#[from] remote_settings::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Settings file is not a json object")]
    SettingsNotObject,
    #[error("Could not get path to settings file")]
    SettingsPathError,
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
            inner: serde_json::from_str(&file)
                .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new())),
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

    pub fn set(
        &mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Result<(), Error> {
        self.inner
            .as_object_mut()
            .ok_or(Error::SettingsNotObject)?
            .insert(key.into(), value.into());

        Ok(())
    }

    pub fn get(&self, key: impl AsRef<str>) -> Option<&serde_json::Value> {
        self.inner.get(key.as_ref())
    }

    pub fn remove(&mut self, key: impl AsRef<str>) -> Result<(), Error> {
        self.inner
            .as_object_mut()
            .ok_or(Error::SettingsNotObject)?
            .remove(key.as_ref());

        Ok(())
    }

    pub fn get_mut(&mut self, key: impl Into<String>) -> Option<&mut serde_json::Value> {
        self.inner.get_mut(key.into())
    }

    pub fn get_mut_settings(&mut self) -> Option<&mut serde_json::Map<String, serde_json::Value>> {
        self.inner.as_object_mut()
    }

    pub fn get_setting(&self) -> Option<&serde_json::Map<String, serde_json::Value>> {
        self.inner.as_object()
    }

    pub fn to_inner(self) -> serde_json::Value {
        self.inner
    }
}
