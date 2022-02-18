use anyhow::{Context, Result};
use directories::BaseDirs;
use std::{fs, path::PathBuf};

pub struct Settings {
    inner: serde_json::Value,
}

impl Settings {
    pub fn path() -> Result<PathBuf> {
        let base_dirs = BaseDirs::new().context("Failed to get base dirs")?;

        let settings_path_1 = base_dirs.config_dir().join("fig").join("settings.json");
        if settings_path_1.exists() {
            return Ok(settings_path_1);
        }

        let settings_path_2 = base_dirs.home_dir().join(".fig").join("settings.json");
        if settings_path_2.exists() {
            return Ok(settings_path_2);
        }

        Err(anyhow::anyhow!("Could not find settings file"))
    }

    pub fn load() -> Result<Self> {
        let settings_path = BaseDirs::new()
            .context("Could not get home dir")?
            .home_dir()
            .join(".fig")
            .join("settings.json");

        let settings_file = fs::read_to_string(settings_path)?;

        Ok(Self {
            inner: serde_json::from_str(&settings_file)?,
        })
    }

    pub fn save(&self) -> Result<()> {
        let settings_path = BaseDirs::new()
            .context("Could not get home dir")?
            .home_dir()
            .join(".fig")
            .join("settings.json");

        fs::write(settings_path, serde_json::to_string_pretty(&self.inner)?)?;
        Ok(())
    }

    pub fn set(&mut self, key: impl Into<String>, value: serde_json::Value) -> Result<()> {
        self.inner
            .as_object_mut()
            .ok_or_else(|| anyhow::anyhow!("Settings is not an object"))?
            .insert(key.into(), value);

        Ok(())
    }

    pub fn get(&self, key: impl Into<String>) -> Option<&serde_json::Value> {
        self.inner
            .get("settings")
            .and_then(|settings| settings.get(key.into()))
    }

    pub fn get_mut(&mut self, key: impl Into<String>) -> Option<&mut serde_json::Value> {
        self.inner
            .get_mut("settings")
            .and_then(|settings| settings.get_mut(key.into()))
    }

    pub fn get_mut_settings(&mut self) -> Option<&mut serde_json::Map<String, serde_json::Value>> {
        self.inner.as_object_mut()
    }

    pub fn get_setting(&self) -> Option<&serde_json::Map<String, serde_json::Value>> {
        self.inner.as_object()
    }
}

pub fn set_value(key: impl Into<String>, value: serde_json::Value) -> Result<()> {
    let mut settings = Settings::load()?;
    settings.set(key, value)?;
    settings.save()?;
    Ok(())
}

pub fn get_value(key: impl Into<String>) -> Result<Option<serde_json::Value>> {
    let settings = Settings::load()?;
    Ok(settings.get(key).cloned())
}
