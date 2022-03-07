use anyhow::{Context, Result};
use clap::{ArgGroup, Args, Subcommand};
use crossterm::style::Stylize;
use fig_ipc::command::{open_ui_element, restart_settings_listener};
use fig_proto::local::UiElement;
use serde_json::json;
use std::process::Command;

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
                println!(
                    "\n{}\nFig might not be running, to launch Fig run: {}\n",
                    "Unable to connect to Fig".bold(),
                    "fig launch".magenta()
                )
            };
        }

        match self.cmd {
            Some(SettingsSubcommands::Init) => {
                let res = restart_settings_listener().await;

                if res.is_err() {
                    print_connection_error!();
                    return res;
                } else {
                    println!("\nSettings listener restarted.\n");
                }
            }
            Some(SettingsSubcommands::Docs) => {
                println!("→ Opening Fig docs...\n");
                let success = Command::new("open")
                    .arg("https://fig.io/docs/support/settings/")
                    .status()?
                    .success();
                if !success {
                    let msg = "Could not open settings file.";
                    println!("{}", msg);
                    anyhow::bail!(msg);
                }
            }
            Some(SettingsSubcommands::Open) => {
                let path = fig_settings::settings::settings_path()
                    .context("Could not get settings path")?;
                if !Command::new("open").arg(path).status()?.success() {
                    anyhow::bail!("Could not open settings file.");
                }
            }
            None => match &self.key {
                Some(key) => match (&self.value, self.delete) {
                    (None, false) => match fig_settings::settings::get_value(key)? {
                        Some(value) => {
                            println!("{}: {}", key, serde_json::to_string_pretty(&value)?);
                        }
                        None => {
                            println!("No value associated with {}.", key);
                        }
                    },
                    (Some(value_str), false) => {
                        let value =
                            serde_json::from_str(value_str).unwrap_or_else(|_| json!(value_str));
                        let remote_result = fig_settings::settings::set_value(key, value).await?;
                        if remote_result.is_err() {
                            println!("Error syncing settings.");
                        }
                        println!("Successfully updated settings");
                    }
                    (None, true) => {
                        let remote_result = fig_settings::settings::remove_value(key).await?;
                        if remote_result.is_err() {
                            println!("Error syncing settings.");
                        }
                        println!("Successfully updated settings");
                    }
                    _ => {}
                },
                None => {
                    #[cfg(target_os = "macos")]
                    launch_fig()?;

                    let res = open_ui_element(UiElement::MissionControl).await;
                    if res.is_err() {
                        print_connection_error!();
                        return res;
                    }
                }
            },
        }
        Ok(())
    }
}
