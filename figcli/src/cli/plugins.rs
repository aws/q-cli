use std::time::Duration;

use anyhow::{bail, Context, Result};
use clap::Subcommand;
use fig_ipc::{connect_timeout, send_recv_message};
use fig_proto::daemon::{
    daemon_response::Response, sync_command::SyncType, sync_response::SyncStatus, DaemonResponse,
};
use fig_settings::api_host;
use reqwest::Url;

use super::OutputFormat;

#[derive(Debug, Subcommand)]
pub enum PluginsSubcommands {
    /// Sync the current plugins
    Sync,
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
        #[clap(long, short, arg_enum, default_value = "plain")]
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
                println!();

                let spinner =
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
                        fig_proto::daemon::new_sync_message(SyncType::Plugins),
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
                            spinner.stop_with_message("✔️ Successfully synced plugins".into());
                            println!();
                            println!();
                        }
                        SyncStatus::Error => {
                            spinner.stop_with_message("❌ Failed to sync plugins".into());
                            bail!(sync_result.error().to_string());
                        }
                    },
                    _ => {
                        spinner.stop_with_message("❌ Failed to sync plugins".into());
                        bail!("Could not get diagnostics from daemon");
                    }
                }

                Ok(())
            }
            PluginsSubcommands::Add { plugin } => {
                println!("Installing plugin {}", plugin);
                bail!("Not implemented");
            }
            PluginsSubcommands::Remove { plugin } => {
                println!("Removing plugin {}", plugin);
                bail!("Not implemented");
            }
            PluginsSubcommands::List { format, .. } => {
                let api_host = api_host();
                let url = Url::parse(&format!("{api_host}/plugins/all"))?;

                let json: serde_json::Value =
                    reqwest::get(url).await?.error_for_status()?.json().await?;

                if let Some(object) = json.as_object() {
                    if let Some(plugins) = object.get("plugins") {
                        if format == &OutputFormat::Plain {
                            if let Some(plugins) = plugins.as_array() {
                                for plugin in plugins {
                                    if let Some(name) = plugin.get("name") {
                                        if let Some(name) = name.as_str() {
                                            println!("{}", name);
                                        }
                                    }
                                }
                            } else {
                                bail!("Plugins in response is not an array");
                            }
                        } else {
                            println!("{}", serde_json::to_string(&plugins)?);
                        }
                    } else {
                        bail!("Could not find plugins in response");
                    }
                } else {
                    bail!("Response is not an object");
                }

                Ok(())
            }
        }
    }
}
