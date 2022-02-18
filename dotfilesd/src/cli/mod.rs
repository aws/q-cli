//! CLI functionality

pub mod app;
pub mod auth;
pub mod debug;
pub mod diagnostics;
pub mod doctor;
pub mod hook;
pub mod init;
pub mod installation;
pub mod internal;
pub mod invite;
pub mod issue;
pub mod plugins;
pub mod settings;
pub mod sync;
pub mod theme;
pub mod tips;
pub mod tweet;
pub mod util;

use crate::{
    cli::{installation::InstallComponents, util::open_url},
    daemon::daemon,
    util::shell::{Shell, When},
};

use anyhow::Result;
use clap::{ArgEnum, IntoApp, Parser, Subcommand};
use crossterm::style::Stylize;
use std::process::exit;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ArgEnum)]
pub enum OutputFormat {
    Plain,
    Json,
}

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
        /// Don't confirm automatic installation.
        #[clap(long)]
        no_confirm: bool,
        /// Force installation of the dotfiles
        #[clap(long)]
        force: bool,
    },
    #[clap(subcommand)]
    /// Interact with the desktop app
    App(app::AppSubcommand),
    #[clap(subcommand)]
    /// Hook commands
    Hook(hook::HookSubcommand),
    #[clap(subcommand)]
    /// Debug Fig
    Debug(debug::DebugSubcommand),
    /// Customize appearance & behavior
    Settings(settings::SettingsArgs),
    #[clap(subcommand)]
    /// Enable/disable fig tips
    Tips(tips::TipsSubcommand),
    /// Uninstall dotfiles
    Uninstall {
        /// Uninstall only the daemon
        #[clap(long)]
        daemon: bool,
        /// Uninstall only the dotfiles
        #[clap(long)]
        dotfiles: bool,
        /// Don't confirm automatic removal.
        #[clap(long)]
        no_confirm: bool,
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
    Diagnostic {
        #[clap(long, short, arg_enum, default_value = "plain")]
        format: OutputFormat,
    },
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
    /// Get or set theme
    Theme { theme: Option<String> },
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
    /// Generate the completion spec for Fig
    GenerateFigCompleation,
    #[clap(subcommand)]
    Internal(internal::InternalSubcommand),
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
                CliRootCommands::Install {
                    daemon,
                    dotfiles,
                    no_confirm,
                    force,
                } => {
                    let install_components = if daemon || dotfiles {
                        let mut install_components = InstallComponents::empty();
                        install_components.set(InstallComponents::DAEMON, daemon);
                        install_components.set(InstallComponents::DOTFILES, dotfiles);
                        install_components
                    } else {
                        InstallComponents::all()
                    };

                    installation::install_cli(install_components, no_confirm, force)
                }
                CliRootCommands::Uninstall {
                    daemon,
                    dotfiles,
                    no_confirm,
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

                    installation::uninstall_cli(uninstall_components, no_confirm)
                }
                CliRootCommands::Update { no_confirm } => {
                    installation::update_cli(no_confirm).await
                }
                CliRootCommands::Tips(tips_subcommand) => tips_subcommand.execute().await,
                CliRootCommands::Daemon => daemon().await,
                CliRootCommands::Diagnostic { format } => {
                    diagnostics::diagnostics_cli(format).await
                }
                CliRootCommands::Init { shell, when } => init::shell_init_cli(&shell, &when).await,
                CliRootCommands::Sync => sync::sync_cli().await,
                CliRootCommands::Login { refresh } => auth::login_cli(refresh).await,
                CliRootCommands::Logout => auth::logout_cli().await,
                CliRootCommands::User => auth::user_info_cli().await,
                CliRootCommands::Doctor => doctor::doctor_cli().await,
                CliRootCommands::Invite => invite::invite_cli().await,
                CliRootCommands::Tweet => tweet::tweet_cli(),
                CliRootCommands::App(app_subcommand) => app_subcommand.execute().await,
                CliRootCommands::Hook(hook_subcommand) => hook_subcommand.execute().await,
                CliRootCommands::Theme { theme } => theme::theme_cli(theme),
                CliRootCommands::Settings(settings_args) => settings_args.execute().await,
                CliRootCommands::Debug(debug_subcommand) => debug_subcommand.execute().await,
                CliRootCommands::Issue { description } => issue::issue_cli(description).await,
                CliRootCommands::Plugins(plugins_subcommand) => plugins_subcommand.execute().await,
                CliRootCommands::GenerateFigCompleation => {
                    println!("{}", Cli::generation_fig_compleations());
                    Ok(())
                }
                CliRootCommands::Internal(internal_subcommand) => {
                    internal_subcommand.execute().await
                }
            },
            // Root command
            None => root_command().await,
        };

        if let Err(e) = result {
            eprintln!("{:?}", e);
            exit(1);
        }
    }

    fn generation_fig_compleations() -> String {
        let mut cli = Cli::command();

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

async fn root_command() -> Result<()> {
    // Check if Fig is running
    match fig_ipc::command::open_ui_element(fig_proto::local::UiElement::MissionControl).await {
        Ok(_) => {}
        Err(_) => {
            let url = "https://dotfiles.com/";
            if open_url(url).is_err() {
                println!("{}", url.underlined());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn debug_assert() {
        Cli::command().debug_assert();
    }
}
