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

    pub fn set(key: impl Into<String>, value: serde_json::Value) -> Result<()> {
        let mut settings = Self::load()?;
        let settings_map = settings
            .get_mut_settings()
            .ok_or(anyhow::anyhow!("Could not load settings"))?;
        settings_map.insert(key.into(), value);
        settings.save()?;
        Ok(())
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

    pub fn get_mut_settings(&mut self) -> Option<&mut serde_json::Map<String, serde_json::Value>> {
        self.inner.as_object_mut()
    }

    pub fn get_setting(&self) -> Option<&serde_json::Map<String, serde_json::Value>> {
        self.inner.as_object()
    }
}
