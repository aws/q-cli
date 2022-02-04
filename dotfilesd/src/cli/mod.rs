//! CLI functionality

pub mod auth;
pub mod diagnostics;
pub mod doctor;
pub mod init;
pub mod installation;
pub mod invite;
pub mod issue;
pub mod plugins;
pub mod sync;
pub mod tweet;
pub mod util;

use self::{init::When, installation::InstallComponents, util::open_url};
use crate::daemon::daemon;
use crate::util::shell::Shell;
use anyhow::Result;
use clap::{IntoApp, Parser, Subcommand};
use crossterm::style::Stylize;
use std::process::exit;

#[derive(Debug, Subcommand)]
pub enum CliRootCommands {
    /// Install dotfiles
    Install {
        /// Install only the daemon
        #[clap(long, conflicts_with = "dotfiles")]
        daemon: bool,
        /// Install only the dotfiles
        #[clap(long)]
        dotfiles: bool,
    },
    /// Uninstall dotfiles
    Uninstall {
        /// Uninstall only the daemon
        #[clap(long)]
        daemon: bool,
        /// Uninstall only the dotfiles
        #[clap(long)]
        dotfiles: bool,
        /// Uninstall only the binary
        #[clap(long)]
        binary: bool,
    },
    /// Update dotfiles
    Update {
        /// Force update
        #[clap(long, short = 'y')]
        no_confirm: bool,
    },
    /// Run the daemon
    Daemon,
    /// Run diagnostic tests
    Diagnostic,
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
    /// Invite friends to Fig
    Invite,
    /// Tweet about Fig
    Tweet,
    /// Create a new Github issue
    Issue {
        /// Issue description
        description: Vec<String>,
    },
    /// Login to dotfiles
    Login {
        #[clap(long, short)]
        refresh: bool,
    },
    /// Logout of dotfiles
    Logout,
    /// Details about the current user
    User,
    /// Check Fig is properly configured
    Doctor,
    /// Plugins management
    #[clap(subcommand)]
    Plugins(plugins::PluginsSubcommand),
    /// Prompt the if there is new version of dotfiles
    Prompt,
    /// Generate the completion spec for Fig
    GenerateFigCompleation,
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
                CliRootCommands::Install { daemon, dotfiles } => {
                    let install_components = if daemon || dotfiles {
                        let mut install_components = InstallComponents::empty();
                        install_components.set(InstallComponents::DAEMON, daemon);
                        install_components.set(InstallComponents::DOTFILES, dotfiles);
                        install_components
                    } else {
                        InstallComponents::all()
                    };

                    installation::install_cli(install_components)
                }
                CliRootCommands::Uninstall {
                    daemon,
                    dotfiles,
                    binary,
                } => {
                    let uninstall_components = if daemon || dotfiles || binary {
                        let mut uninstall_components = InstallComponents::empty();
                        uninstall_components.set(InstallComponents::DAEMON, daemon);
                        uninstall_components.set(InstallComponents::DOTFILES, dotfiles);
                        uninstall_components.set(InstallComponents::BINARY, binary);
                        uninstall_components
                    } else {
                        InstallComponents::all()
                    };

                    installation::uninstall_cli(uninstall_components)
                }
                CliRootCommands::Update { no_confirm } => installation::update_cli(no_confirm),
                CliRootCommands::Daemon => daemon().await,
                CliRootCommands::Diagnostic => diagnostics::diagnostics_cli().await,
                CliRootCommands::Init { shell, when } => init::shell_init_cli(&shell, &when).await,
                CliRootCommands::Sync => sync::sync_cli().await,
                CliRootCommands::Login { refresh } => auth::login_cli(refresh).await,
                CliRootCommands::Logout => auth::logout_cli().await,
                CliRootCommands::User => auth::user_info_cli().await,
                CliRootCommands::Doctor => doctor::doctor_cli().await,
                CliRootCommands::Invite => invite::invite_cli().await,
                CliRootCommands::Tweet => tweet::tweet_cli(),
                CliRootCommands::Issue { description } => issue::issue_cli(description).await,
                CliRootCommands::Plugins(plugins_subcommand) => plugins_subcommand.execute().await,
                CliRootCommands::Prompt => sync::prompt_cli().await,
                CliRootCommands::GenerateFigCompleation => {
                    println!("{}", Cli::generation_fig_compleations());
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

    fn generation_fig_compleations() -> String {
        let mut cli = Cli::into_app();

        let mut buffer = Vec::new();

        clap_complete::generate(
            clap_complete_fig::Fig,
            &mut cli,
            env!("CARGO_PKG_NAME"),
            &mut buffer,
        );

        String::from_utf8_lossy(&buffer).into()
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
