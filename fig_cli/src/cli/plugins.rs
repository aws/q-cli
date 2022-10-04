use clap::Subcommand;
use crossterm::style::Stylize;
use eyre::{
    bail,
    Result,
};
use fig_api_client::plugins::{
    all_plugins,
    installed_plugins,
    unique_plugin,
};
use fig_request::Request;

use super::OutputFormat;
use crate::util::dialoguer_theme;

#[derive(Debug, Subcommand)]
pub enum PluginsSubcommands {
    /// Sync the current plugins (this will not update plugins that are already installed)
    Sync,
    /// Update the installed plugins
    Update,
    /// Install a specific plugin from the plugin store
    Add {
        /// The plugin to install
        plugin: String,
    },
    /// Uninstall a specific plugin
    Remove {
        /// The plugin to uninstall
        plugin: String,
    },
    /// List all plugins available in the plugin store
    List {
        /// Only list plugins that are installed
        #[arg(long, short)]
        installed: bool,
        /// Fields to include in the output
        #[arg(long, value_delimiter = ',', default_value = "name,displayName,icon,description")]
        fields: Vec<String>,
        /// The output format
        #[arg(long, short, value_enum, default_value_t)]
        format: OutputFormat,
    },
    /// Info about a specific plugin
    Info {
        /// The plugin to get info about
        plugin: String,
        /// Fields to include in the output
        #[arg(long, value_delimiter = ',', default_value = "name,displayName,description")]
        fields: Vec<String>,
        /// The output format
        #[arg(long, short, value_enum, default_value_t)]
        format: OutputFormat,
    },
    /// Configure a specific plugin
    Configure {
        /// The plugin to configure
        plugin: Option<String>,
        /// The configuration options to set
        config: Option<String>,
        // The value to set the configuration option to
        // value: Option<String>,
    },
}

impl PluginsSubcommands {
    pub async fn execute(&self) -> Result<()> {
        match self {
            PluginsSubcommands::Sync => {
                let mut spinner = spinners::Spinner::new(spinners::Spinners::Dots, "Syncing plugins".into());

                let fetch_result = fig_sync::plugins::fetch_installed_plugins(false).await;

                match fetch_result {
                    Ok(_) => {
                        spinner.stop_with_message(format!("{} Successfully synced plugins\n", "✔️".green()));
                        Ok(())
                    },
                    Err(err) => {
                        spinner.stop_with_message(format!("{} Failed to sync plugins\n", "✖️".red()));
                        Err(err.into())
                    },
                }
            },
            PluginsSubcommands::Update => {
                let mut spinner = spinners::Spinner::new(spinners::Spinners::Dots, "Syncing plugins".into());

                let fetch_result = fig_sync::plugins::fetch_installed_plugins(true).await;

                match fetch_result {
                    Ok(_) => {
                        spinner.stop_with_message(format!("{} Successfully update plugins\n", "✔️".green()));
                        Ok(())
                    },
                    Err(err) => {
                        spinner.stop_with_message(format!("{} Failed to update plugins\n", "✖️".red()));
                        Err(err.into())
                    },
                }
            },
            PluginsSubcommands::Add { plugin } => {
                let mut spinner =
                    spinners::Spinner::new(spinners::Spinners::Arc, format!("Installing plugin {plugin}"));

                let response = Request::post(format!("/dotfiles/plugins/add/{plugin}"))
                    .auth()
                    .send()
                    .await;

                match response {
                    Ok(_) => {
                        spinner.stop_with_message(format!("{} Successfully installed plugin\n", "✔️".green()));
                        println!(
                            "Run {} to start using the plugin in the current shell",
                            "fig source".magenta()
                        );
                        Ok(())
                    },
                    Err(err) => {
                        spinner.stop_with_message(format!("{} Failed to install plugin\n", "✘".red(),));
                        eyre::bail!(err)
                    },
                }
            },
            PluginsSubcommands::Remove { plugin } => {
                let mut spinner =
                    spinners::Spinner::new(spinners::Spinners::Arc, format!("Removing plugin {}", plugin));

                let response = Request::post(format!("/dotfiles/plugins/remove/{plugin}"))
                    .auth()
                    .send()
                    .await;

                match response {
                    Ok(_) => {
                        spinner.stop_with_message(format!("{} Successfully removed plugin\n", "✔️".green()));
                        println!(
                            "Run {} to stop using the plugin in the current shell",
                            "fig source".magenta()
                        );
                        Ok(())
                    },
                    Err(err) => {
                        spinner.stop_with_message(format!("{} Failed to remove plugin\n", "✘".red(),));
                        eyre::bail!(err)
                    },
                }
            },
            PluginsSubcommands::List {
                format,
                installed,
                fields,
            } => {
                let plugins = if *installed {
                    installed_plugins(fields).await?
                } else {
                    all_plugins(fields).await?
                };

                match format {
                    OutputFormat::Plain => {
                        for plugin in plugins {
                            for (key, value) in plugin {
                                println!("{key}: {value}");
                            }
                            println!();
                        }
                    },
                    OutputFormat::Json => {
                        println!("{}", serde_json::to_string(&plugins)?);
                    },
                    OutputFormat::JsonPretty => {
                        println!("{}", serde_json::to_string_pretty(&plugins)?);
                    },
                }

                Ok(())
            },
            PluginsSubcommands::Info { fields, format, plugin } => {
                let data = unique_plugin(plugin, fields).await?;
                match format {
                    OutputFormat::Plain => {
                        for (key, value) in data {
                            println!("{key}: {value}");
                        }
                    },
                    OutputFormat::Json => {
                        println!("{}", serde_json::to_string(&data)?);
                    },
                    OutputFormat::JsonPretty => {
                        println!("{}", serde_json::to_string_pretty(&data)?);
                    },
                }
                Ok(())
            },
            PluginsSubcommands::Configure { plugin, config, .. } => {
                let (plugin, print_name) = match plugin {
                    Some(plugin) => (plugin.clone(), true),
                    None => {
                        let plugins = installed_plugins(["name"]).await?;

                        let idx = dialoguer::Select::with_theme(&dialoguer_theme())
                            .with_prompt("Select plugin to configure")
                            .items(
                                &plugins
                                    .iter()
                                    .filter_map(|entry| {
                                        entry
                                            .get("displayName")
                                            .or_else(|| entry.get("name"))
                                            .and_then(|name| name.as_str().map(String::from))
                                    })
                                    .collect::<Vec<_>>(),
                            )
                            .default(0)
                            .interact()?;

                        (plugins[idx].get("name").unwrap().as_str().unwrap().to_string(), false)
                    },
                };

                let plugin = unique_plugin(plugin, ["name", "displayName", "raw"]).await?;

                let plugin_name = plugin
                    .get("displayName")
                    .or_else(|| plugin.get("name"))
                    .and_then(|name| name.as_str().map(String::from));

                if print_name {
                    match plugin_name {
                        Some(plugin_name) => {
                            println!("Configuring {plugin_name}");
                        },
                        None => {
                            println!("Configuring plugin");
                        },
                    }
                }

                let configuration = match plugin["raw"].as_str() {
                    Some(raw) => {
                        let json = serde_json::from_str::<serde_json::Value>(raw)?;
                        match json {
                            serde_json::Value::Object(mut map) => match map.remove("configuration") {
                                Some(configuration) => match configuration {
                                    serde_json::Value::Array(configuration) => configuration,
                                    _ => bail!("Configuration is not an array"),
                                },
                                None => bail!("Plugin does not have a configuration"),
                            },
                            _ => bail!("Plugin raw is not a JSON object"),
                        }
                    },
                    None => bail!("Could not find raw config for plugin"),
                };

                let flat_config = configuration
                    .into_iter()
                    .filter_map(|value| match value {
                        serde_json::Value::Object(map) => Some(map),
                        _ => None,
                    })
                    .flat_map(|mut map| match map.remove("configuration") {
                        Some(child) => match child {
                            serde_json::Value::Array(array) => array
                                .into_iter()
                                .flat_map(|value| match value {
                                    serde_json::Value::Object(map) => Some(map),
                                    _ => None,
                                })
                                .collect::<Vec<_>>(),
                            _ => vec![],
                        },
                        None => vec![map],
                    })
                    .collect::<Vec<_>>();

                if flat_config.is_empty() {
                    println!("No configuration found");
                    return Ok(());
                }

                let config = match config {
                    Some(config) => Some(&**config),
                    None => {
                        let idx = dialoguer::Select::with_theme(&dialoguer_theme())
                            .with_prompt("Select configuration to set")
                            .items(
                                &flat_config
                                    .iter()
                                    .filter_map(|entry| {
                                        entry
                                            .get("displayName")
                                            .or_else(|| entry.get("name"))
                                            .and_then(|name| name.as_str().map(String::from))
                                    })
                                    .collect::<Vec<_>>(),
                            )
                            .default(0)
                            .interact()?;

                        flat_config[idx].get("name").and_then(|name| name.as_str())
                    },
                };

                let config = match config {
                    Some(config) => config,
                    None => {
                        println!("No configuration found");
                        return Ok(());
                    },
                };

                let config_entry = flat_config
                    .iter()
                    .find(|entry| entry.get("name").map_or(false, |name| name == config));

                let entry = match config_entry {
                    Some(config_entry) => config_entry,
                    None => {
                        println!("No configuration found");
                        return Ok(());
                    },
                };

                let name: String = entry
                    .get("displayName")
                    .or_else(|| entry.get("name"))
                    .and_then(|name| name.as_str().map(|name| name.into()))
                    .unwrap_or_else(|| "config".into());

                match entry.get("interface").and_then(|i| i.as_str()) {
                    Some("text" | "textarea") => {
                        dialoguer::Input::<String>::with_theme(&dialoguer_theme())
                            .with_prompt(format!("Set {name} to"))
                            .interact_text()?;
                    },
                    Some("toggle" | "checkbox") => {
                        dialoguer::Select::with_theme(&dialoguer_theme())
                            .with_prompt(format!("Set {name}"))
                            .items(&["true", "false"])
                            .interact()?;
                    },
                    Some("multi-text") => {
                        println!("Multi-text is not yet supported");
                    },
                    Some("select") => {
                        println!("Select is not yet supported");
                    },
                    Some("multiselect") => {
                        println!("Multi-select is not yet supported");
                    },
                    Some(interface) => {
                        println!("Unsupported interface: {interface}")
                    },
                    None => println!("Unsupported interface: None"),
                }

                Ok(())
            },
        }
    }
}
