use anyhow::{anyhow, Context, Result};
use clap::{ArgGroup, Args, Subcommand};
use crossterm::style::Stylize;
use fig_auth::is_logged_in;
use fig_ipc::command::{open_ui_element, restart_settings_listener};
use fig_proto::local::UiElement;
use fig_settings::remote_settings::RemoteSettings;
use globset::Glob;
use serde_json::json;
use std::{
    io::Write,
    process::{exit, Command},
};
use time::format_description::well_known::Rfc3339;

use super::{util::app_not_running_message, OutputFormat};
use crate::util::{launch_fig, LaunchOptions};

#[derive(Debug, Subcommand)]
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
        #[clap(short, long)]
        remote: bool,
        /// Format of the output
        #[clap(long, short, arg_enum, default_value_t)]
        format: OutputFormat,
    },
}

#[derive(Debug, Args)]
#[clap(subcommand_negates_reqs = true)]
#[clap(args_conflicts_with_subcommands = true)]
#[clap(group(ArgGroup::new("vals").requires("key").args(&["value", "delete", "format"])))]
pub struct SettingsArgs {
    #[clap(subcommand)]
    cmd: Option<SettingsSubcommands>,
    /// key
    key: Option<String>,
    /// value
    value: Option<String>,
    #[clap(long, short)]
    /// Delete a value
    delete: bool,
    #[clap(long, short, arg_enum, default_value_t)]
    /// Format of the output
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
                    }
                    Err(err) => {
                        print_connection_error!();
                        Err(err)
                    }
                }
            }
            Some(SettingsSubcommands::Docs) => {
                println!("→ Opening Fig docs...\n");

                let success = Command::new("open")
                    .arg("https://fig.io/docs/support/settings/")
                    .status()?
                    .success();

                match success {
                    true => Ok(()),
                    false => Err(anyhow!("Could not open settings file")),
                }
            }
            Some(SettingsSubcommands::Open) => {
                let path = fig_settings::settings::settings_path()
                    .context("Could not get settings path")?;
                match Command::new("open").arg(path).status()?.success() {
                    true => Ok(()),
                    false => Err(anyhow!("Could not open settings file")),
                }
            }
            Some(SettingsSubcommands::Sync) => {
                let RemoteSettings {
                    settings,
                    updated_at,
                } = fig_settings::remote_settings::get_settings().await?;

                let path = fig_settings::settings::settings_path()
                    .context("Could not get settings path")?;

                let mut settings_file = std::fs::File::create(&path)?;
                let settings_json = serde_json::to_string_pretty(&settings)?;
                settings_file.write_all(settings_json.as_bytes())?;

                if let Ok(updated_at) = updated_at.format(&Rfc3339) {
                    fig_settings::state::set_value("settings.updatedAt", json!(updated_at)).ok();
                }

                Ok(())
            }
            Some(SettingsSubcommands::All { remote, format }) => {
                let settings = if remote {
                    fig_settings::remote_settings::get_settings()
                        .await?
                        .settings
                } else {
                    fig_settings::settings::local_settings()?.to_inner()
                };

                match format {
                    OutputFormat::Plain => {
                        if let Some(map) = settings.as_object() {
                            for (key, value) in map {
                                println!("{} = {}", key, value);
                            }
                        } else {
                            println!("Settings is empty");
                        }
                    }
                    OutputFormat::Json => println!("{}", serde_json::to_string(&settings)?),
                    OutputFormat::JsonPretty => {
                        println!("{}", serde_json::to_string_pretty(&settings)?)
                    }
                }

                Ok(())
            }
            None => match &self.key {
                Some(key) => match (&self.value, self.delete) {
                    (None, false) => match fig_settings::settings::get_value(key)? {
                        Some(value) => {
                            match self.format {
                                OutputFormat::Plain => match value.as_str() {
                                    Some(value) => println!("{}", value),
                                    None => println!("{:#}", value),
                                },
                                OutputFormat::Json => {
                                    println!("{}", value)
                                }
                                OutputFormat::JsonPretty => {
                                    println!("{:#}", value)
                                }
                            }
                            Ok(())
                        }
                        None => match self.format {
                            OutputFormat::Plain => {
                                Err(anyhow::anyhow!("No value associated with {}", key))
                            }
                            OutputFormat::Json | OutputFormat::JsonPretty => {
                                println!("null");
                                Ok(())
                            }
                        },
                    },
                    (Some(value_str), false) => {
                        let value =
                            serde_json::from_str(value_str).unwrap_or_else(|_| json!(value_str));
                        let remote_result = fig_settings::settings::set_value(key, value).await;
                        match remote_result {
                            Ok(()) => {
                                println!("Successfully set setting");
                                Ok(())
                            }
                            Err(err) => match err {
                                fig_settings::Error::RemoteSettingsError(
                                    fig_settings::remote_settings::Error::AuthError,
                                ) => {
                                    eprintln!("You are not logged in to Fig");
                                    eprintln!("Run {} to login", "fig login".magenta().bold());
                                    exit(1);
                                }
                                err => Err(err.into()),
                            },
                        }
                    }
                    (None, true) => {
                        let glob = Glob::new(key)
                            .context("Could not create glob")?
                            .compile_matcher();

                        let map = fig_settings::settings::get_map()?
                            .context("Could not get settings map")?;

                        let keys_to_remove = map
                            .keys()
                            .filter(|key| glob.is_match(key))
                            .collect::<Vec<_>>();

                        match keys_to_remove.len() {
                            0 => {
                                return Err(anyhow::anyhow!("No settings found matching {}", key));
                            }
                            1 => {
                                println!("Removing: {:?}", keys_to_remove[0]);
                            }
                            _ => {
                                println!("Removing:");
                                for key in &keys_to_remove {
                                    println!("  - {key}");
                                }
                            }
                        }

                        let futures = keys_to_remove
                            .into_iter()
                            .map(fig_settings::settings::remove_value)
                            .collect::<Vec<_>>();

                        let join = futures::future::join_all(futures).await;

                        for result in join {
                            if let Err(err) = result {
                                match err {
                                    fig_settings::Error::RemoteSettingsError(
                                        fig_settings::remote_settings::Error::AuthError,
                                    ) => {
                                        eprintln!("You are not logged in to Fig");
                                        eprintln!("Run {} to login", "fig login".magenta().bold());
                                        exit(1);
                                    }
                                    err => return Err(err.into()),
                                }
                            }
                        }

                        Ok(())
                    }
                    _ => Ok(()),
                },
                None => {
                    println!();
                    launch_fig(LaunchOptions::new().wait_for_activation().verbose())?;
                    println!();

                    if is_logged_in() {
                        match open_ui_element(UiElement::Settings).await {
                            Ok(()) => Ok(()),
                            Err(err) => {
                                print_connection_error!();
                                Err(err.context("Could not open settings"))
                            }
                        }
                    } else {
                        Ok(())
                    }
                }
            },
        }
    }
}
