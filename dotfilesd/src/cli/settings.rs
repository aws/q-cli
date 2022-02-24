use crate::util::{fig_dir, settings::Settings};

use anyhow::{anyhow, Result};
use clap::{ArgGroup, Args, Subcommand};
use crossterm::style::Stylize;
use fig_ipc::command::{open_ui_element, restart_settings_listener};
use fig_proto::local::UiElement;
use serde_json::json;
use std::process::Command;

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
                let path = fig_dir()
                    .map(|p| p.join("settings.json"))
                    .ok_or(anyhow!("Could not find fig directory"))?;
                if !Command::new("open").arg(path).status()?.success() {
                    anyhow::bail!("Could not open settings file.");
                }
            }
            None => match &self.key {
                Some(key) => {
                    let mut settings = Settings::load()?;
                    let settings_map = settings
                        .get_mut_settings()
                        .ok_or(anyhow!("Could not load settings"))?;
                    match (&self.value, self.delete) {
                        (None, false) => match settings_map.get(key) {
                            Some(value) => {
                                println!("{}: {}", key, serde_json::to_string_pretty(value)?);
                            }
                            None => {
                                println!("No value associated with {}.", key);
                            }
                        },
                        (Some(value), false) => {
                            settings_map.insert(key.into(), json!(value));
                            settings.save()?;
                            println!("Successfully updated settings");
                        }
                        (None, true) => {
                            settings_map.remove(key);
                            settings.save()?;
                            println!("Successfully updated settings");
                        }
                        _ => {}
                    }
                }
                None => {
                    let res = open_ui_element(UiElement::Settings).await;
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
