use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Subcommand;

use crate::plugins::{download_plugin, manifest::Plugin};

fn read_plugin_from_file(path: impl AsRef<Path>) -> Result<Plugin> {
    let raw = std::fs::read_to_string(path)?;
    let mut plugin: Plugin = toml::from_str(&raw)?;
    plugin.normalize();
    Ok(plugin)
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
    Download {
        plugin_file: PathBuf,
    },
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
            PluginsSubcommand::Download { plugin_file } => {
                let plugin: Plugin = read_plugin_from_file(plugin_file)?;
                download_plugin(&plugin).await?;
            }
        }

        Ok(())
    }
}
