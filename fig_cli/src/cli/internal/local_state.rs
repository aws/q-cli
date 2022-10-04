use std::process::Command;

use clap::{
    ArgGroup,
    Args,
    Subcommand,
};
use crossterm::style::Stylize;
use eyre::{
    eyre,
    Result,
};
use fig_ipc::local::restart_settings_listener;
use fig_util::directories;
use serde_json::json;

use crate::cli::OutputFormat;

#[derive(Debug, Subcommand)]
pub enum LocalStateSubcommand {
    /// Reload the state listener
    Init,
    /// Open the state file
    Open,
    /// List all the settings
    All {
        #[arg(long, short, value_enum, default_value_t)]
        format: OutputFormat,
    },
}

#[derive(Debug, Args)]
#[command(subcommand_negates_reqs = true)]
#[command(args_conflicts_with_subcommands = true)]
#[command(group(ArgGroup::new("vals").requires("key").args(&["value", "delete", "format"])))]
pub struct LocalStateArgs {
    #[command(subcommand)]
    cmd: Option<LocalStateSubcommand>,
    /// Key of the state
    key: Option<String>,
    /// Value of the state
    value: Option<String>,
    #[arg(long, short)]
    /// Delete the state
    delete: bool,
    #[arg(long, short, value_enum, default_value_t)]
    /// Format of the output
    format: OutputFormat,
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
                },
                Err(err) => {
                    print_connection_error!();
                    Err(err.into())
                },
            },
            Some(LocalStateSubcommand::Open) => {
                let path = directories::state_path()?;
                match Command::new("open").arg(path).status()?.success() {
                    true => Ok(()),
                    false => Err(eyre!("Could not open state file")),
                }
            },
            Some(LocalStateSubcommand::All { format }) => {
                let map = fig_settings::state::local_settings()?.inner;

                match format {
                    OutputFormat::Plain => {
                        for (key, value) in map {
                            println!("{key} = {value}");
                        }
                    },
                    OutputFormat::Json => println!("{}", serde_json::to_string(&map)?),
                    OutputFormat::JsonPretty => {
                        println!("{}", serde_json::to_string_pretty(&map)?)
                    },
                }

                Ok(())
            },
            None => match &self.key {
                Some(key) => match (&self.value, self.delete) {
                    (None, false) => match fig_settings::state::get_value(key)? {
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
                    (None, true) => {
                        fig_settings::state::remove_value(key)?;
                        println!("Successfully updated state");
                        Ok(())
                    },
                    (Some(value), false) => {
                        let value: serde_json::Value = serde_json::from_str(value).unwrap_or_else(|_| json!(value));
                        fig_settings::state::set_value(key, value)?;
                        println!("Successfully updated state");
                        Ok(())
                    },
                    (Some(_), true) => Err(eyre!("Cannot delete a value with a value")),
                },
                None => Err(eyre!("No key specified")),
            },
        }
    }
}
