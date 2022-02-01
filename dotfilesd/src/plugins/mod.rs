use crate::util::shell::Shell;

use self::manifest::{Plugin, PluginType};

pub mod download;
pub mod init;
pub mod lock;
pub mod manifest;

use anyhow::{Context, Result};

pub async fn download_plugin(plugin: &Plugin) -> Result<()> {
    let plugin_data_dir = download::plugin_data_dir().context("Failed to get source folder")?;
    let plugin_dir = plugin_data_dir.join(&plugin.metadata.name);

    if plugin_dir.exists() {
        return Err(anyhow::anyhow!("Plugin already exists"));
    }

    let metadata = match plugin.metadata.plugin_type {
        PluginType::Shell => {
            if let Some(shell) = &plugin.installation.shell {
                shell.source.download_source(&plugin_dir).await?
            } else {
                return Err(anyhow::anyhow!("No installation found for plugin"));
            }
        }
        PluginType::Theme => todo!(),
        PluginType::Special => todo!(),
    };

    let mut lock_file = lock::LockData::load()
        .await
        .unwrap_or_else(|_| lock::LockData::new());

    let mut entry = lock::LockEntry::new(plugin.metadata.name.clone());
    entry.download_metadata = Some(metadata);
    entry.version = plugin.metadata.version.clone();

    match &plugin.metadata.shells {
        Some(shells) => {
            for shell in shells {
                let locked = plugin
                    .installation
                    .shell
                    .as_ref()
                    .map(|s| s.default_install.as_ref().map(|i| i.lock(&plugin_dir, shell).unwrap()))
                    // .flatten()
                    .flatten();

                if let Some(locked) = locked {
                    entry.shell_install.insert(*shell, locked);
                }
            }
        }
        None => {
            for shell in Shell::all() {
                let locked = plugin
                    .installation
                    .shell
                    .as_ref()
                    .map(|s| s.default_install.as_ref().map(|i| i.lock(&plugin_dir, shell).ok()))
                    .flatten()
                    .flatten();

                if let Some(locked) = locked {
                    entry.shell_install.insert(*shell, locked);
                }
            }
        }
    }

    lock_file.add_entry(entry);
    lock_file.save().await.unwrap();

    Ok(())
}
