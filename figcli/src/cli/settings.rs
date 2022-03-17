use anyhow::{anyhow, Context, Result};
use clap::{ArgGroup, Args, Subcommand};
use fig_ipc::command::{open_ui_element, restart_settings_listener};
use fig_proto::local::UiElement;
use serde_json::json;
use std::process::Command;

use super::util::app_not_running_message;
use crate::util::launch_fig;

#[derive(Debug, Subcommand)]
pub enum SettingsSubcommands {
    /// Reload the settings listener
    Init,
    /// Get the settings documentation
    Docs,
    /// Open the settings file
    Open,
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
                        println!("\nSettings listener restarted.\n");
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
                    false => Err(anyhow!("Could not open settings file.")),
                }
            }
            Some(SettingsSubcommands::Open) => {
                let path = fig_settings::settings::settings_path()
                    .context("Could not get settings path")?;
                match Command::new("open").arg(path).status()?.success() {
                    true => Ok(()),
                    false => Err(anyhow!("Could not open settings file.")),
                }
            }
            None => match &self.key {
                Some(key) => match (&self.value, self.delete) {
                    (None, false) => match fig_settings::settings::get_value(key)? {
                        Some(value) => {
                            println!("{}", serde_json::to_string_pretty(&value)?);
                            Ok(())
                        }
                        None => Err(anyhow::anyhow!("No value associated with {}.", key)),
                    },
                    (Some(value_str), false) => {
                        let value =
                            serde_json::from_str(value_str).unwrap_or_else(|_| json!(value_str));
                        let remote_result = fig_settings::settings::set_value(key, value).await?;
                        match remote_result {
                            Ok(()) => {
                                println!("Error syncing settings.");
                                Ok(())
                            }
                            Err(_) => Err(anyhow!("Successfully updated settings")),
                        }
                    }
                    (None, true) => {
                        let remote_result = fig_settings::settings::remove_value(key).await?;
                        match remote_result {
                            Ok(()) => {
                                println!("Successfully updated settings");
                                Ok(())
                            }
                            Err(_) => Err(anyhow!("Error syncing settings.")),
                        }
                    }
                    _ => Ok(()),
                },
                None => {
                    println!();
                    launch_fig()?;
                    println!();

                    match open_ui_element(UiElement::Settings).await {
                        Ok(()) => Ok(()),
                        Err(err) => {
                            print_connection_error!();
                            Err(err.context("Could not open settings"))
                        }
                    }
                }
            },
        }
    }
}
