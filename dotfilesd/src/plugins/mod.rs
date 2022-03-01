pub mod api;
pub mod download;
pub mod init;
pub mod install;
pub mod lock;
pub mod manifest;

use crate::{
    plugins::manifest::{Plugin, PluginType},
    util::shell::Shell,
};

use anyhow::{bail, Context, Result};

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
        PluginType::Theme => bail!("Themes are not supported yet"),
        PluginType::Special => bail!("Special plugins are not supported yet"),
    };

    let mut lock_file = lock::LockData::load()
        .await
        .unwrap_or_else(|_| lock::LockData::new());

    let mut entry = lock::LockEntry::new(plugin.metadata.name.clone());
    entry.download_metadata = Some(metadata);
    entry.version = plugin.metadata.version.clone();

    let default_install = plugin
        .installation
        .shell
        .as_ref()
        .and_then(|shell_install| shell_install.default_install.clone())
        .unwrap_or_default();

    let mut install_plugin = |shell: &Shell| {
        let install = if let Some(s) = plugin
            .installation
            .shell
            .as_ref()
            .and_then(|shell_install| shell_install.per_shell.get(shell))
        {
            default_install.merge(s)
        } else {
            default_install.clone()
        };

        let locked = install.lock(&plugin_dir, shell, &plugin.metadata.name);

        if let Ok(locked) = locked {
            entry.shell_install.insert(*shell, locked);
        }
    };

    match &plugin.metadata.shells {
        Some(shells) => {
            for shell in shells {
                install_plugin(shell);
            }
        }
        None => {
            for shell in Shell::all() {
                install_plugin(shell);
            }
        }
    }

    lock_file.add_entry(entry);
    lock_file.save().await.unwrap();

    Ok(())
}
