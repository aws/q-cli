use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs::File, path::PathBuf, io::{Read, Write}};

/// Get the path to the config folder
fn get_config_folder() -> Result<PathBuf> {
    let mut path =
        dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Could not get config directory"))?;
    path.push("dotfiles");
    Ok(path)
}

#[derive(Debug)]
pub struct ConfigFile {
    file: File,
}

impl ConfigFile {
    pub fn load() -> Result<Self> {
        let config_folder_path = get_config_folder()?;

        // Create the config folder if it doesn't exist
        if !config_folder_path.exists() {
            std::fs::create_dir_all(&config_folder_path)?;
        }

        let config_file_path = config_folder_path.join("config.toml");

        let file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(&config_file_path)?;

        Ok(Self { file })
    }

    pub fn data(&mut self) -> Result<ConfigData> {
        let mut data = String::new();
        self.file.read_to_string(&mut data)?;

        Ok(toml::from_str(&data)?)
    }

    pub fn save(&mut self, data: impl AsRef<ConfigData>) -> Result<()> {
        let data = toml::to_string(data.as_ref())?;
        self.file.write_all(data.as_bytes())?;

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ConfigData {
    pub user_token: Option<String>,
    pub autoupdate: Option<bool>,
    pub last_update_check: Option<u64>,
}
