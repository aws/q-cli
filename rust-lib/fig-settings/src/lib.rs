pub mod remote_settings;
pub mod settings;
pub mod state;

use anyhow::Result;
use std::{fs, path::PathBuf};

pub struct LocalJson {
    inner: serde_json::Value,
    path: PathBuf,
}

impl LocalJson {
    pub fn load(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let settings_file = fs::read_to_string(&path)?;

        Ok(Self {
            inner: serde_json::from_str(&settings_file)?,
            path,
        })
    }

    pub fn save(&self) -> Result<()> {
        fs::write(&self.path, serde_json::to_string_pretty(&self.inner)?)?;
        Ok(())
    }

    pub fn set(
        &mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Result<()> {
        self.inner
            .as_object_mut()
            .ok_or_else(|| anyhow::anyhow!("Settings is not an object"))?
            .insert(key.into(), value.into());

        Ok(())
    }

    pub fn get(&self, key: impl AsRef<str>) -> Option<&serde_json::Value> {
        self.inner.get(key.as_ref())
    }

    pub fn remove(&mut self, key: impl AsRef<str>) -> Result<()> {
        self.inner
            .as_object_mut()
            .ok_or_else(|| anyhow::anyhow!("Settings is not an object"))?
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
}
