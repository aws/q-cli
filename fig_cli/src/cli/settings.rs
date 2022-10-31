use clap::{
    ArgGroup,
    Args,
    Subcommand,
};
use eyre::{
    Result,
    WrapErr,
};
use fig_api_client::settings;
use fig_ipc::local::{
    open_ui_element,
    restart_settings_listener,
};
use fig_proto::local::UiElement;
use fig_request::auth::is_logged_in;
use fig_util::desktop::{
    launch_fig_desktop,
    LaunchArgs,
};
use fig_util::directories;
use globset::Glob;
use serde_json::json;

use super::OutputFormat;
use crate::util::app_not_running_message;

#[derive(Debug, Subcommand, PartialEq, Eq)]
pub enum SettingsSubcommands {
    /// Reload the settings listener
    Init,
    /// Get the settings documentation
    Docs,
    /// Open the settings file
    Open,
    /// Sync the current settings
    Sync,
    /// List all the settings
    All {
        /// List the remote settings
        #[arg(long, short)]
        remote: bool,
        /// Format of the output
        #[arg(long, short, value_enum, default_value_t)]
        format: OutputFormat,
    },
}

#[derive(Debug, Args, PartialEq, Eq)]
#[command(subcommand_negates_reqs = true)]
#[command(args_conflicts_with_subcommands = true)]
#[command(group(ArgGroup::new("vals").requires("key").args(&["value", "delete", "format"])))]
pub struct SettingsArgs {
    #[command(subcommand)]
    cmd: Option<SettingsSubcommands>,
    /// key
    key: Option<String>,
    /// value
    value: Option<String>,
    /// Delete a value
    #[arg(long, short)]
    delete: bool,
    /// Format of the output
    #[arg(long, short, value_enum, default_value_t)]
    format: OutputFormat,
}

impl SettingsArgs {
    pub async fn execute(&self) -> Result<()> {
        macro_rules! print_connection_error {
            () => {
                println!("{}", app_not_running_message());
            };
        }

        match self.cmd {
            Some(SettingsSubcommands::Init) => {
                let res = restart_settings_listener().await;

                match res {
                    Ok(()) => {
                        println!("\nSettings listener restarted\n");
                        Ok(())
                    },
                    Err(err) => {
                        print_connection_error!();
                        Err(err.into())
                    },
                }
            },
            Some(SettingsSubcommands::Docs) => {
                println!("→ Opening Fig docs...");
                fig_util::open_url("https://fig.io/docs/support/settings/")?;
                Ok(())
            },
            Some(SettingsSubcommands::Open) => {
                let mut url = String::from("file://");
                url.push_str(
                    &directories::settings_path()
                        .context("Could not get settings path")?
                        .to_string_lossy(),
                );
                fig_util::open_url(url)?;
                Ok(())
            },
            Some(SettingsSubcommands::Sync) => {
                settings::sync().await?;
                Ok(())
            },
            Some(SettingsSubcommands::All { remote, format }) => {
                let settings = if remote {
                    settings::get().await?.settings
                } else {
                    fig_settings::settings::local_settings()?.inner
                };

                match format {
                    OutputFormat::Plain => {
                        for (key, value) in settings {
                            println!("{key} = {value}");
                        }
                    },
                    OutputFormat::Json => println!("{}", serde_json::to_string(&settings)?),
                    OutputFormat::JsonPretty => {
                        println!("{}", serde_json::to_string_pretty(&settings)?)
                    },
                }

                Ok(())
            },
            None => match &self.key {
                Some(key) => match (&self.value, self.delete) {
                    (None, false) => match fig_settings::settings::get_value(key)? {
                        Some(value) => {
                            match self.format {
                                OutputFormat::Plain => match value.as_str() {
                                    Some(value) => println!("{value}"),
                                    None => println!("{value:#}"),
                                },
                                OutputFormat::Json => println!("{value}"),
                                OutputFormat::JsonPretty => println!("{value:#}"),
                            }
                            Ok(())
                        },
                        None => match self.format {
                            OutputFormat::Plain => Err(eyre::eyre!("No value associated with {key}")),
                            OutputFormat::Json | OutputFormat::JsonPretty => {
                                println!("null");
                                Ok(())
                            },
                        },
                    },
                    (Some(value_str), false) => {
                        let value = serde_json::from_str(value_str).unwrap_or_else(|_| json!(value_str));
                        settings::update(key, value).await?;
                        println!("Successfully set setting");
                        Ok(())
                    },
                    (None, true) => {
                        let glob = Glob::new(key).context("Could not create glob")?.compile_matcher();
                        let map = fig_settings::settings::get_map()?;
                        let keys_to_remove = map.keys().filter(|key| glob.is_match(key)).collect::<Vec<_>>();

                        match keys_to_remove.len() {
                            0 => {
                                return Err(eyre::eyre!("No settings found matching {key}"));
                            },
                            1 => {
                                println!("Removing {:?}", keys_to_remove[0]);
                            },
                            _ => {
                                println!("Removing:");
                                for key in &keys_to_remove {
                                    println!("  - {key}");
                                }
                            },
                        }

                        let futures = keys_to_remove.into_iter().map(settings::delete).collect::<Vec<_>>();

                        let join = futures::future::join_all(futures).await;

                        for result in join {
                            if let Err(err) = result {
                                println!("{err}");
                            }
                        }

                        Ok(())
                    },
                    _ => Ok(()),
                },
                None => {
                    launch_fig_desktop(LaunchArgs {
                        wait_for_socket: true,
                        open_dashboard: false,
                        immediate_update: true,
                        verbose: true,
                    })?;

                    if is_logged_in() {
                        match open_ui_element(UiElement::Settings, None).await {
                            Ok(()) => Ok(()),
                            Err(err) => {
                                print_connection_error!();
                                Err(err.into())
                            },
                        }
                    } else {
                        Ok(())
                    }
                },
            },
        }
    }
}
