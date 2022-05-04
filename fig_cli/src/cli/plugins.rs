use std::time::Duration;

use anyhow::{bail, Context, Result};
use clap::Subcommand;
use crossterm::style::Stylize;
use fig_ipc::{connect_timeout, send_recv_message};
use fig_proto::daemon::{
    daemon_response::Response, sync_command::SyncType, sync_response::SyncStatus, DaemonResponse,
};
use fig_settings::api_host;
use reqwest::{Client, Url};

use super::OutputFormat;
use crate::util::api::handle_fig_response;

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
        /// The output format
        #[clap(long, short, arg_enum, default_value_t)]
        format: OutputFormat,
        /// Only list plugins that are installed
        #[clap(long, short)]
        installed: bool,
    },
}

impl PluginsSubcommands {
    pub async fn execute(&self) -> Result<()> {
        match self {
            PluginsSubcommands::Sync => {
                let mut spinner =
                    spinners::Spinner::new(spinners::Spinners::Dots, "Syncing plugins".into());

                // Get diagnostics from the daemon
                let socket_path = fig_ipc::daemon::get_daemon_socket_path();

                if !socket_path.exists() {
                    bail!("Could not find daemon socket, run `fig doctor` to diagnose");
                }

                let mut conn = match connect_timeout(&socket_path, Duration::from_secs(1)).await {
                    Ok(connection) => connection,
                    Err(_) => {
                        bail!("Could not connect to daemon socket, run `fig doctor` to diagnose");
                    }
                };

                let diagnostic_response_result: Option<fig_proto::daemon::DaemonResponse> =
                    send_recv_message(
                        &mut conn,
                        fig_proto::daemon::new_sync_message(SyncType::PluginClone),
                        Duration::from_secs(10),
                    )
                    .await
                    .context("Could not get diagnostics from daemon")?;

                match diagnostic_response_result {
                    Some(DaemonResponse {
                        response: Some(Response::Sync(sync_result)),
                        ..
                    }) => match sync_result.status() {
                        SyncStatus::Ok => {
                            spinner.stop_with_message(format!(
                                "{} Successfully synced plugins\n",
                                "✔️".green()
                            ));
                        }
                        SyncStatus::Error => {
                            spinner.stop_with_message(format!(
                                "{} Failed to sync plugins\n",
                                "✖️".red()
                            ));
                            bail!(sync_result.error().to_string());
                        }
                    },
                    _ => {
                        spinner
                            .stop_with_message(format!("{} Failed to sync plugins\n", "✖️".red()));
                        bail!("Could not get diagnostics from daemon");
                    }
                }

                Ok(())
            }
            PluginsSubcommands::Update => {
                let mut spinner =
                    spinners::Spinner::new(spinners::Spinners::Dots, "Updating plugins".into());

                // Get diagnostics from the daemon
                let socket_path = fig_ipc::daemon::get_daemon_socket_path();

                if !socket_path.exists() {
                    bail!("Could not find daemon socket, run `fig doctor` to diagnose");
                }

                let mut conn = match connect_timeout(&socket_path, Duration::from_secs(1)).await {
                    Ok(connection) => connection,
                    Err(_) => {
                        bail!("Could not connect to daemon socket, run `fig doctor` to diagnose");
                    }
                };

                let diagnostic_response_result: Option<fig_proto::daemon::DaemonResponse> =
                    send_recv_message(
                        &mut conn,
                        fig_proto::daemon::new_sync_message(SyncType::PluginUpdate),
                        Duration::from_secs(10),
                    )
                    .await
                    .context("Could not get diagnostics from daemon")?;

                match diagnostic_response_result {
                    Some(DaemonResponse {
                        response: Some(Response::Sync(sync_result)),
                        ..
                    }) => match sync_result.status() {
                        SyncStatus::Ok => {
                            spinner.stop_with_message(format!(
                                "{} Successfully updated plugins\n",
                                "✔".green()
                            ));
                        }
                        SyncStatus::Error => {
                            spinner.stop_with_message(format!(
                                "{} Failed to updated plugins\n",
                                "✖️".red()
                            ));
                            bail!(sync_result.error().to_string());
                        }
                    },
                    _ => {
                        spinner.stop_with_message(format!(
                            "{} Failed to updated plugins\n",
                            "✖️".red()
                        ));
                        bail!("Could not get diagnostics from daemon");
                    }
                }

                Ok(())
            }
            PluginsSubcommands::Add { plugin } => {
                let mut spinner = spinners::Spinner::new(
                    spinners::Spinners::Arc,
                    format!("Installing plugin {}", plugin),
                );

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
                        spinner.stop_with_message(format!(
                            "{} Successfully installed plugin\n",
                            "✔️".green()
                        ));
                        println!(
                            "Run {} to start using the plugin in the current shell",
                            "fig source".magenta()
                        );
                        Ok(())
                    }
                    Err(err) => {
                        spinner
                            .stop_with_message(
                                format!("{} Failed to install plugin\n", "✘".red(),),
                            );
                        Err(err)
                    }
                }
            }
            PluginsSubcommands::Remove { plugin } => {
                let mut spinner = spinners::Spinner::new(
                    spinners::Spinners::Arc,
                    format!("Removing plugin {}", plugin),
                );

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
                        spinner.stop_with_message(format!(
                            "{} Successfully removed plugin\n",
                            "✔️".green()
                        ));
                        println!(
                            "Run {} to stop using the plugin in the current shell",
                            "fig source".magenta()
                        );
                        Ok(())
                    }
                    Err(err) => {
                        spinner
                            .stop_with_message(format!("{} Failed to remove plugin\n", "✘".red(),));
                        Err(err)
                    }
                }
            }
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
                                    }
                                    OutputFormat::Json => {
                                        println!("{}", serde_json::to_string(&json)?);
                                    }
                                    OutputFormat::JsonPretty => {
                                        println!("{}", serde_json::to_string_pretty(&json)?)
                                    }
                                }
                                Ok(())
                            } else {
                                bail!("Could not find plugins in response");
                            }
                        } else {
                            println!("{}", json);
                            bail!("Response is not an object");
                        }
                    }
                    Err(err) => Err(err),
                }
            }
        }
    }
}
