use anyhow::{anyhow, Context, Result};
use clap::{ArgGroup, Args, Subcommand};
use fig_auth::is_logged_in;
use fig_ipc::command::{open_ui_element, restart_settings_listener};
use fig_proto::local::UiElement;
use fig_settings::remote_settings::RemoteSettings;
use serde_json::json;
use std::{io::Write, process::Command};
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
        #[clap(short, long)]
        remote: bool,
        #[clap(long, short, arg_enum, default_value_t)]
        format: OutputFormat,
    },
}

#[derive(Debug, Args)]
#[clap(subcommand_negates_reqs = true)]
#[clap(args_conflicts_with_subcommands = true)]
#[clap(group(ArgGroup::new("vals").requires("key").args(&["value", "delete"])))]
pub struct SettingsArgs {
    #[clap(subcommand)]
    cmd: Option<SettingsSubcommands>,
    /// key
    key: Option<String>,
    /// value
    value: Option<String>,
    #[clap(long, short)]
    /// delete
    delete: bool,
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
                            println!("{}", serde_json::to_string_pretty(&value)?);
                            Ok(())
                        }
                        None => Err(anyhow::anyhow!("No value associated with {}", key)),
                    },
                    (Some(value_str), false) => {
                        let value =
                            serde_json::from_str(value_str).unwrap_or_else(|_| json!(value_str));
                        let remote_result = fig_settings::settings::set_value(key, value).await;
                        match remote_result {
                            Ok(Ok(())) => {
                                println!("Successfully set setting");
                                Ok(())
                            }
                            Ok(Err(err)) => {
                                eprintln!("Error setting setting:");
                                Err(err)
                            }
                            Err(err) => {
                                eprintln!("Error setting setting:");
                                Err(err)
                            }
                        }
                    }
                    (None, true) => {
                        let remote_result = fig_settings::settings::remove_value(key).await;
                        match remote_result {
                            Ok(Ok(())) => {
                                println!("Successfully removed settings");
                                Ok(())
                            }
                            Ok(Err(err)) => {
                                eprintln!("Error removing setting, it may already be removed");
                                Err(err)
                            }
                            Err(err) => {
                                eprintln!("Error syncing setting");
                                Err(err)
                            }
                        }
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
