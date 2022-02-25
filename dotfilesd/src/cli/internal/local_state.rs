use anyhow::Result;
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
    /// key
    key: Option<String>,
    /// value
    value: Option<String>,
    #[clap(long, short)]
    /// delete
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
            Some(LocalStateSubcommand::Init) => {
                let res = restart_settings_listener().await;

                if res.is_err() {
                    print_connection_error!();
                    return res;
                } else {
                    println!("\nState listener restarted.\n");
                }
            }
            Some(LocalStateSubcommand::Open) => {
                let path = fig_settings::state::state_path()?;
                if !Command::new("open").arg(path).status()?.success() {
                    anyhow::bail!("Could not open state file.");
                }
            }
            None => match &self.key {
                Some(key) => match (&self.value, self.delete) {
                    (None, false) => match fig_settings::state::get_value(key)? {
                        Some(value) => {
                            println!("{}: {}", key, serde_json::to_string_pretty(&value)?);
                        }
                        None => {
                            println!("No value associated with {}.", key);
                        }
                    },
                    (None, true) => {
                        fig_settings::state::remove_value(key)?;
                        println!("Successfully updated state");
                    }
                    (Some(value), false) => {
                        fig_settings::state::set_value(key, json!(value))?;
                        println!("Successfully updated state");
                    }

                    (Some(_), true) => {
                        eprintln!("Cannot delete a value with a value.");
                    }
                },
                None => {
                    eprintln!("{}", "No key specified.".red());
                }
            },
        }
        Ok(())
    }
}
