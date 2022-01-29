//! CLI functionality

pub mod auth;
pub mod doctor;
pub mod init;
pub mod installation;
pub mod plugins;
pub mod sync;
pub mod util;

use self::{init::When, util::open_url};
use crate::daemon::daemon;
use crate::util::shell::Shell;
use anyhow::Result;
use clap::{Parser, Subcommand};
use crossterm::style::Stylize;
use std::process::exit;

#[derive(Debug, Subcommand)]
pub enum CliRootCommands {
    /// Install dotfiles
    Install,
    /// Uninstall dotfiles
    Uninstall,
    /// Update dotfiles
    Update {
        /// Force update
        #[clap(long, short = 'y')]
        no_confirm: bool,
    },
    /// Run the daemon
    Daemon,
    /// Generate the dotfiles for the given shell
    Init {
        /// The shell to generate the dotfiles for
        #[clap(arg_enum)]
        shell: Shell,
        /// When to generate the dotfiles for
        #[clap(arg_enum)]
        when: When,
    },
    /// Sync your latest dotfiles
    Sync,
    /// Login to dotfiles
    Login {
        #[clap(long, short)]
        refresh: bool,
    },
    /// Logout of dotfiles
    Logout,
    /// Details about the current user
    User,
    /// Doctor
    Doctor,
    #[clap(subcommand)]
    Plugins(plugins::PluginsSubcommand),
    Test,
}

#[derive(Debug, Parser)]
#[clap(version, about)]
pub struct Cli {
    #[clap(subcommand)]
    pub subcommand: Option<CliRootCommands>,
}

impl Cli {
    pub async fn execute(self) {
        let result = match self.subcommand {
            Some(subcommand) => match subcommand {
                CliRootCommands::Install => installation::install_cli(),
                CliRootCommands::Uninstall => installation::uninstall_cli(),
                CliRootCommands::Update { no_confirm } => installation::update_cli(no_confirm),
                CliRootCommands::Daemon => daemon().await,
                CliRootCommands::Init { shell, when } => init::shell_init_cli(&shell, &when),
                CliRootCommands::Sync => sync::sync_cli().await,
                CliRootCommands::Login { refresh } => auth::login_cli(refresh).await,
                CliRootCommands::Logout => auth::logout_cli().await,
                CliRootCommands::User => auth::user_info_cli().await,
                CliRootCommands::Doctor => doctor::doctor_cli(),
                CliRootCommands::Plugins(plugins_subcommand) => plugins_subcommand.execute().await,
                CliRootCommands::Test => {
                    crate::daemon::websocket::connect_to_fig_websocket()
                        .await
                        .unwrap();
                    Ok(())
                }
            },
            // Root command
            None => root_command(),
        };

        if let Err(e) = result {
            eprintln!("{:?}", e);
            exit(1);
        }
    }
}

fn root_command() -> Result<()> {
    // Open the default browser to the homepage
    let url = "https://dotfiles.com/";
    if open_url(url).is_err() {
        println!("{}", url.underlined());
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use clap::IntoApp;

    use super::*;

    #[test]
    fn debug_assert() {
        Cli::into_app().debug_assert();
    }
}
