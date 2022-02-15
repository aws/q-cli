use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Subcommand;

use crate::plugins::{
    download::plugin_data_dir, download_plugin, lock::LockData, manifest::Plugin,
};

fn read_plugin_from_file(path: impl AsRef<Path>) -> Result<Plugin> {
    let raw = std::fs::read_to_string(path)?;
    let mut plugin: Plugin = toml::from_str(&raw)?;
    plugin.normalize();
    Ok(plugin)
}

async fn get_plugin_from_repo(name: impl AsRef<str>) -> Result<Plugin> {
    let raw = reqwest::get(&format!(
        "https://raw.githubusercontent.com/withfig/plugins/main/plugins/{}.toml",
        name.as_ref()
    ))
    .await?
    .error_for_status()
    .context("Failed to get plugin from repo")?
    .text()
    .await?;

    let mut plugin: Plugin = toml::from_str(&raw)?;
    plugin.normalize();
    Ok(plugin)
}

async fn remove_plugin(name: impl AsRef<str>) -> Result<()> {
    let plugin_dir = plugin_data_dir().context("Failed to get plugin data dir")?;
    let plugin_path = plugin_dir.join(name.as_ref());

    let mut lock_data = LockData::load().await?;

    lock_data
        .get_entries_mut()
        .retain(|entry| entry.name != name.as_ref());

    tokio::fs::remove_dir_all(plugin_path).await?;

    lock_data.save().await?;

    Ok(())
}

#[derive(Debug, Subcommand)]
pub enum PluginsSubcommand {
    Info {
        plugin_file: PathBuf,
        #[clap(long, short, conflicts_with = "quiet")]
        verbose: bool,
        /// Quiet
        #[clap(long, short, conflicts_with = "verbose")]
        quiet: bool,
    },
    Add {
        /// Name of the plugin to download
        plugin: String,
        #[clap(long, short)]
        local: bool,
        #[clap(long, short)]
        force: bool,
    },
    Remove {
        /// Name of the plugin to remove
        plugin: String,
    },
    List,
}

impl PluginsSubcommand {
    pub async fn execute(&self) -> Result<()> {
        match self {
            PluginsSubcommand::Info {
                plugin_file,
                verbose,
                quiet,
            } => {
                // Read from the plugin
                let plugin: Plugin = match read_plugin_from_file(plugin_file) {
                    Ok(v) => v,
                    Err(err) => {
                        if *quiet {
                            return Err(anyhow::anyhow!(""));
                        }

                        if *verbose {
                            return Err(anyhow::anyhow!("{:#?}", err));
                        }

                        return Err(anyhow::anyhow!("{}", err));
                    }
                };

                if let Err(e) = plugin.validate() {
                    if *quiet {
                        return Err(anyhow::anyhow!(""));
                    }

                    if *verbose {
                        return Err(anyhow::anyhow!("{:#?}", e));
                    }

                    return Err(anyhow::anyhow!("{}", e));
                }

                if !quiet {
                    println!("{:#?}", plugin);
                }
            }
            PluginsSubcommand::Add {
                local,
                plugin,
                force,
            } => {
                if *force {
                    remove_plugin(plugin).await.ok();
                }

                let plugin = match local {
                    true => {
                        let path = Path::new(&plugin);
                        if path.exists() {
                            let plugin = read_plugin_from_file(path)?;
                            if let Err(e) = plugin.validate() {
                                return Err(anyhow::anyhow!("{}", e));
                            }
                            plugin
                        } else {
                            return Err(anyhow::anyhow!("Plugin does not exist"));
                        }
                    }
                    false => {
                        let plugin = get_plugin_from_repo(plugin).await?;
                        if let Err(e) = plugin.validate() {
                            return Err(anyhow::anyhow!("{}", e));
                        }
                        plugin
                    }
                };

                download_plugin(&plugin).await?;
            }
            PluginsSubcommand::Remove { plugin } => {
                remove_plugin(plugin).await?;
            }
            PluginsSubcommand::List => {
                let lock_file = LockData::load().await?;
                for plugin in lock_file.get_entries() {
                    println!("{}", plugin.name);
                }
            }
        }

        Ok(())
    }
}
