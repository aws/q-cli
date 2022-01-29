use std::path::PathBuf;

use anyhow::Result;
use clap::Subcommand;

use crate::plugins::manifest::Plugin;

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
                let toml_file = std::fs::read_to_string(plugin_file)?;

                let plugin: Plugin = match toml::from_str(&toml_file) {
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
        }

        Ok(())
    }
}
