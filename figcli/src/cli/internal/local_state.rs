use anyhow::{anyhow, Context, Result};
use clap::{ArgGroup, Args, Subcommand};
use crossterm::style::Stylize;
use fig_ipc::command::restart_settings_listener;
use serde_json::json;
use std::process::Command;

#[derive(Debug, Subcommand)]
pub enum LocalStateSubcommand {
    /// Reload the state listener
    Init,
    /// Open the state file
    Open,
}

#[derive(Debug, Args)]
#[clap(subcommand_negates_reqs = true)]
#[clap(args_conflicts_with_subcommands = true)]
#[clap(group(ArgGroup::new("vals").requires("key").args(&["value", "delete"])))]
pub struct LocalStateArgs {
    #[clap(subcommand)]
    cmd: Option<LocalStateSubcommand>,
    /// Key of the state
    key: Option<String>,
    /// Value of the state
    value: Option<String>,
    #[clap(long, short)]
    /// Delete the state
    delete: bool,
}

impl LocalStateArgs {
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
            Some(LocalStateSubcommand::Init) => match restart_settings_listener().await {
                Ok(()) => {
                    println!("\nState listener restarted\n");
                    Ok(())
                }
                Err(err) => {
                    print_connection_error!();
                    Err(err)
                }
            },
            Some(LocalStateSubcommand::Open) => {
                let path = fig_settings::state::state_path().context("Could not get state path")?;
                match Command::new("open").arg(path).status()?.success() {
                    true => Ok(()),
                    false => Err(anyhow!("Could not open state file")),
                }
            }
            None => match &self.key {
                Some(key) => match (&self.value, self.delete) {
                    (None, false) => match fig_settings::state::get_value(key)? {
                        Some(value) => {
                            println!("{}", serde_json::to_string_pretty(&value)?);
                            Ok(())
                        }
                        None => Err(anyhow!("No value associated with {}", key)),
                    },
                    (None, true) => {
                        fig_settings::state::remove_value(key)?;
                        println!("Successfully updated state");
                        Ok(())
                    }
                    (Some(value), false) => {
                        let value: serde_json::Value =
                            serde_json::from_str(value).unwrap_or_else(|_| json!(value));
                        fig_settings::state::set_value(key, value)?;
                        println!("Successfully updated state");
                        Ok(())
                    }
                    (Some(_), true) => Err(anyhow!("Cannot delete a value with a value")),
                },
                None => Err(anyhow!("{}", "No key specified")),
            },
        }
    }
}
