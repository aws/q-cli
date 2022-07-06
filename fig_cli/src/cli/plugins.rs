use anyhow::{
    bail,
    Result,
};
use clap::Subcommand;
use crossterm::style::Stylize;
use fig_request::handle_fig_response;
use fig_settings::api_host;
use reqwest::{
    Client,
    Url,
};

use super::OutputFormat;

#[derive(Debug, Subcommand)]
pub enum PluginsSubcommands {
    /// Sync the current plugins (this will not update plugins that are already installed)
    Sync,
    /// Update the installed plugins
    Update,
    /// Install a specific plugin from the plugin store
    Add {
        /// The plugin to install
        #[clap(value_parser)]
        plugin: String,
    },
    /// Uninstall a specific plugin
    Remove {
        /// The plugin to uninstall
        #[clap(value_parser)]
        plugin: String,
    },
    /// List all plugins available in the plugin store
    List {
        /// The output format
        #[clap(long, short, value_enum, value_parser, default_value_t)]
        format: OutputFormat,
        /// Only list plugins that are installed
        #[clap(long, short, value_parser)]
        installed: bool,
    },
}

impl PluginsSubcommands {
    pub async fn execute(&self) -> Result<()> {
        match self {
            PluginsSubcommands::Sync => {
                let mut spinner = spinners::Spinner::new(spinners::Spinners::Dots, "Syncing plugins".into());

                let fetch_result = fig_install::plugins::fetch_installed_plugins(false).await;

                match fetch_result {
                    Ok(_) => {
                        spinner.stop_with_message(format!("{} Successfully synced plugins\n", "✔️".green()));
                    },
                    Err(_) => {
                        spinner.stop_with_message(format!("{} Failed to sync plugins\n", "✖️".red()));
                    },
                }

                Ok(())
            },
            PluginsSubcommands::Update => {
                let mut spinner = spinners::Spinner::new(spinners::Spinners::Dots, "Syncing plugins".into());

                let fetch_result = fig_install::plugins::fetch_installed_plugins(true).await;

                match fetch_result {
                    Ok(_) => {
                        spinner.stop_with_message(format!("{} Successfully update plugins\n", "✔️".green()));
                    },
                    Err(_) => {
                        spinner.stop_with_message(format!("{} Failed to update plugins\n", "✖️".red()));
                    },
                }
                Ok(())
            },
            PluginsSubcommands::Add { plugin } => {
                let mut spinner =
                    spinners::Spinner::new(spinners::Spinners::Arc, format!("Installing plugin {}", plugin));

                let api_host = api_host();
                let url = Url::parse(&format!("{api_host}/dotfiles/plugins/add/{plugin}"))?;

                let token = fig_auth::get_token().await?;

                let response = Client::new()
                    .post(url)
                    .bearer_auth(token)
                    .header("Accept", "application/json")
                    .send()
                    .await?;

                match handle_fig_response(response).await {
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
                        anyhow::bail!(err)
                    },
                }
            },
            PluginsSubcommands::Remove { plugin } => {
                let mut spinner =
                    spinners::Spinner::new(spinners::Spinners::Arc, format!("Removing plugin {}", plugin));

                let api_host = api_host();
                let url = Url::parse(&format!("{api_host}/dotfiles/plugins/remove/{plugin}"))?;

                let token = fig_auth::get_token().await?;

                let response = Client::new()
                    .post(url)
                    .bearer_auth(token)
                    .header("Accept", "application/json")
                    .send()
                    .await?;

                match handle_fig_response(response).await {
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
                        anyhow::bail!(err)
                    },
                }
            },
            PluginsSubcommands::List { format, installed } => {
                let api_host = api_host();
                let url = match installed {
                    false => Url::parse(&format!("{api_host}/plugins/all"))?,
                    true => Url::parse(&format!("{api_host}/dotfiles/plugins"))?,
                };

                let mut request = Client::new().get(url);

                if *installed {
                    let token = fig_auth::get_token().await?;
                    request = request.bearer_auth(token)
                }

                let response = request.send().await?;

                match handle_fig_response(response).await {
                    Ok(response) => {
                        let json: serde_json::Value = response.json().await?;

                        if let Some(object) = json.as_object() {
                            if let Some(plugins) = object.get("plugins") {
                                match format {
                                    OutputFormat::Plain => {
                                        if let Some(plugins) = plugins.as_array() {
                                            for plugin in plugins {
                                                if let Some(name) = plugin.get("name") {
                                                    if let Some(name) = name.as_str() {
                                                        println!("{}", name);
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    OutputFormat::Json => {
                                        println!("{}", serde_json::to_string(&json)?);
                                    },
                                    OutputFormat::JsonPretty => {
                                        println!("{}", serde_json::to_string_pretty(&json)?)
                                    },
                                }
                                Ok(())
                            } else {
                                bail!("Could not find plugins in response");
                            }
                        } else {
                            println!("{}", json);
                            bail!("Response is not an object");
                        }
                    },
                    Err(err) => {
                        anyhow::bail!(err)
                    },
                }
            },
        }
    }
}
