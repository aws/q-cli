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
pub mod source;
pub mod theme;
pub mod tips;
pub mod tweet;
pub mod util;

use crate::{
    cli::{installation::InstallComponents, util::open_url},
    daemon::{daemon, get_daemon},
    util::{
        launch_fig,
        shell::{Shell, When},
    },
};

use anyhow::{Context, Result};
use clap::{ArgEnum, IntoApp, Parser, Subcommand};
use crossterm::style::Stylize;
use fig_ipc::command::open_ui_element;
use fig_proto::local::UiElement;
use std::{fs::File, process::exit, str::FromStr};
use tracing::{debug, level_filters::LevelFilter};

use self::app::AppSubcommand;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ArgEnum)]
pub enum OutputFormat {
    Plain,
    Json,
}

#[derive(Debug, Subcommand)]
pub enum CliRootCommands {
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
    /// Install fig cli comoponents
    Install(internal::InstallArgs),
    /// Uninstall fig
    Uninstall,
    /// Update dotfiles
    Update {
        /// Force update
        #[clap(long, short = 'y')]
        no_confirm: bool,
    },
    /// Run the daemon
    #[clap(hide = true)]
    Daemon,
    /// Run diagnostic tests
    Diagnostic {
        #[clap(long, short, arg_enum, default_value = "plain")]
        format: OutputFormat,
    },
    /// Generate the dotfiles for the given shell
    #[clap(hide = true)]
    Init {
        /// The shell to generate the dotfiles for
        #[clap(arg_enum)]
        shell: Shell,
        /// When to generate the dotfiles for
        #[clap(arg_enum)]
        when: When,
    },
    /// Sync your latest dotfiles
    Source,
    /// Get or set theme
    Theme {
        theme: Option<String>,
    },
    /// Invite friends to Fig
    Invite,
    /// Tweet about Fig
    Tweet,
    /// Create a new Github issue
    Issue {
        /// Force issue creation
        #[clap(long, short = 'f')]
        force: bool,
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
    Doctor {
        #[clap(long)]
        verbose: bool,
        #[clap(long)]
        strict: bool,
        #[clap(long)]
        no_early_exit: bool,
    },
    /// Plugins management
    #[clap(subcommand)]
    Plugins(plugins::PluginsSubcommand),
    /// Generate the completion spec for Fig
    GenerateFigSpec,
    #[clap(subcommand)]
    Internal(internal::InternalSubcommand),
    Launch,
    Quit,
    Restart,
    Alpha,
    Onboarding,
}

#[derive(Debug, Parser)]
#[clap(version, about)]
#[clap(help_template = "
  \x1B[1m███████╗██╗ ██████╗
  ██╔════╝██║██╔════╝
  █████╗  ██║██║  ███╗
  ██╔══╝  ██║██║   ██║
  ██║     ██║╚██████╔╝
  ╚═╝     ╚═╝ ╚═════╝ CLI\x1B[0m

 \x1B[1;90mUsage:\x1B[0;90m fig [command]\x1B[0m

 \x1B[1;95mCommon Subcommands\x1B[0m
╭──────────────────────────────────────────────────╮
│ \x1B[1mdoctor\x1B[0m         \x1B[0;90mCheck Fig is properly configured\x1B[0m  │
│ \x1B[1msettings\x1B[0m       \x1B[0;90mCustomize appearance & behavior\x1B[0m   │
│ \x1B[1missue\x1B[0m          \x1B[0;90mCreate a new GitHub issue\x1B[0m         │
│ \x1B[1mtweet\x1B[0m          \x1B[0;90mTweet about Fig\x1B[0m                   │
│ \x1B[1mupdate\x1B[0m         \x1B[0;90mUpdate Fig\x1B[0m                        │
╰──────────────────────────────────────────────────╯

 \x1B[0;90mFor more info on a specific command, use:\x1B[0m
  > fig help [command]
")]
pub struct Cli {
    #[clap(subcommand)]
    pub subcommand: Option<CliRootCommands>,
}

impl Cli {
    pub async fn execute(self) {
        let env_level = std::env::var("FIG_LOG_LEVEL")
            .ok()
            .and_then(|level| LevelFilter::from_str(&level).ok())
            .unwrap_or(LevelFilter::INFO);

        match self.subcommand {
            Some(CliRootCommands::Daemon) => {
                // The daemon prints all logs to stdout
                tracing_subscriber::fmt()
                    .with_max_level(env_level)
                    .with_line_number(true)
                    .init();
            }
            _ => {
                // All other cli commands print logs to ~/.fig/logs/cli.log
                if let Some(fig_dir) = fig_directories::fig_dir() {
                    let log_path = fig_dir.join("logs").join("cli.log");

                    // Create the log directory if it doesn't exist
                    if !log_path.parent().unwrap().exists() {
                        std::fs::create_dir_all(log_path.parent().unwrap()).ok();
                    }

                    if let Ok(log_file) =
                        File::create(log_path).context("failed to create log file")
                    {
                        tracing_subscriber::fmt()
                            .with_writer(log_file)
                            .with_max_level(env_level)
                            .with_line_number(true)
                            .init();
                    }
                }

                debug!("Command ran: {:?}", std::env::args().collect::<Vec<_>>());
            }
        }

        let result = match self.subcommand {
            Some(subcommand) => match subcommand {
                CliRootCommands::Install(args) => internal::install_cli_from_args(args),
                CliRootCommands::Uninstall => {
                    if fig_ipc::command::uninstall_command().await.is_err() {
                        installation::uninstall_cli(InstallComponents::all())
                    } else {
                        Ok(())
                    }
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
                CliRootCommands::Source => source::source_cli().await,
                CliRootCommands::Login { refresh } => auth::login_cli(refresh).await,
                CliRootCommands::Logout => auth::logout_cli().await,
                CliRootCommands::User => auth::user_info_cli().await,
                CliRootCommands::Doctor {
                    verbose,
                    strict,
                    no_early_exit,
                } => doctor::doctor_cli(verbose, strict, no_early_exit).await,
                CliRootCommands::Invite => invite::invite_cli().await,
                CliRootCommands::Tweet => tweet::tweet_cli(),
                CliRootCommands::App(app_subcommand) => app_subcommand.execute().await,
                CliRootCommands::Hook(hook_subcommand) => hook_subcommand.execute().await,
                CliRootCommands::Theme { theme } => theme::theme_cli(theme).await,
                CliRootCommands::Settings(settings_args) => settings_args.execute().await,
                CliRootCommands::Debug(debug_subcommand) => debug_subcommand.execute().await,
                CliRootCommands::Issue { force, description } => {
                    issue::issue_cli(force, description).await
                }
                CliRootCommands::Plugins(plugins_subcommand) => plugins_subcommand.execute().await,
                CliRootCommands::GenerateFigSpec => {
                    println!("{}", Cli::generation_fig_compleations());
                    Ok(())
                }
                CliRootCommands::Internal(internal_subcommand) => {
                    internal_subcommand.execute().await
                }
                CliRootCommands::Launch => {
                    let app_res = app::launch_fig_cli();
                    match get_daemon() {
                        Ok(d) => d.start(),
                        Err(e) => Err(anyhow::anyhow!(e)),
                    }
                    .ok();
                    app_res
                }
                CliRootCommands::Quit => {
                    let app_res = app::quit_fig().await;
                    let daemon_res = match get_daemon() {
                        Ok(d) => d.stop(),
                        Err(e) => Err(anyhow::anyhow!(e)),
                    };
                    if daemon_res.is_err() {
                        println!("Error stopping Fig daemon");
                    }
                    app_res.or(daemon_res)
                }
                CliRootCommands::Restart => {
                    let app_res = app::restart_fig().await;
                    let daemon_res = match get_daemon() {
                        Ok(d) => d.restart(),
                        Err(e) => Err(anyhow::anyhow!(e)),
                    };
                    if daemon_res.is_err() {
                        println!("Error restarting Fig daemon");
                    }
                    app_res.or(daemon_res)
                }
                CliRootCommands::Alpha => {
                    launch_fig().ok();
                    let res = open_ui_element(UiElement::MissionControl).await;
                    if res.is_ok() {
                        println!("\n→ Opening dotfiles...\n");
                    };
                    res
                }
                CliRootCommands::Onboarding => {
                    let res = AppSubcommand::Onboarding.execute().await;
                    res
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
    #[cfg(target_os = "macos")]
    launch_fig()?;

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
