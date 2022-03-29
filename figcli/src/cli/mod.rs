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
    cli::util::dialoguer_theme,
    daemon::{daemon, get_daemon},
    util::{
        is_app_running, launch_fig,
        shell::{Shell, When},
        LaunchOptions,
    },
};

use anyhow::{Context, Result};
use cfg_if::cfg_if;
use clap::{ArgEnum, IntoApp, Parser, Subcommand};
use std::{fs::File, process::exit, str::FromStr};
use tracing::{debug, level_filters::LevelFilter};

use self::{app::AppSubcommand, plugins::PluginsSubcommands};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ArgEnum)]
pub enum OutputFormat {
    /// Outputs the results as markdown
    Plain,
    /// Outputs the results as JSON
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ArgEnum)]
pub enum Shells {
    /// Bash shell compleations
    Bash,
    /// Fish shell completions
    Fish,
    /// Zsh shell completions
    Zsh,
    /// Fig completion spec
    Fig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ArgEnum)]
pub enum Processes {
    /// Daemon process
    Daemon,
    /// Fig process
    App,
}

#[derive(Debug, Subcommand)]
pub enum CliRootCommands {
    #[clap(subcommand)]
    /// Interact with the desktop app
    App(app::AppSubcommand),
    #[clap(subcommand, hide = true)]
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
    #[clap(hide = true)]
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
        /// The format of the output
        #[clap(long, short, arg_enum, default_value = "plain")]
        format: OutputFormat,
        /// Force limited diagnostic output
        #[clap(long)]
        force: bool,
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
    Source,
    /// Get or set theme
    Theme { theme: Option<String> },
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
    /// Login to Fig
    Login {
        /// Manually refresh the auth token
        #[clap(long, short)]
        refresh: bool,
    },
    /// Logout of Fig
    Logout,
    /// Details about the current user
    User,
    /// Check Fig is properly configured
    Doctor {
        /// Run all doctor tests, with no fixes
        #[clap(long)]
        verbose: bool,
        /// Error on warnings
        #[clap(long)]
        strict: bool,
    },
    /// Generate the completion spec for Fig
    #[clap(hide = true)]
    Completion {
        /// Shell to generate the completion spec for
        #[clap(arg_enum, default_value = "zsh")]
        shell: Shells,
    },
    /// Internal subcommands used for Fig
    #[clap(subcommand, hide = true)]
    Internal(internal::InternalSubcommand),
    /// Launch the Fig desktop app
    Launch,
    /// Quit the Fig desktop app
    Quit,
    /// Restart the Fig desktop app
    Restart {
        /// The process to restart
        #[clap(arg_enum, default_value = "app", hide = true)]
        process: Processes,
    },
    #[clap(hide = true)]
    /// (LEGACY) Old way to launch mission control
    Alpha,
    /// Run the Fig tutorial
    Onboarding,
    /// (LEGACY) Old hook that was being used somewhere
    #[clap(name = "app:running", hide = true)]
    FigAppRunning,
    #[clap(subcommand)]
    Plugins(PluginsSubcommands),
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
                if env_level >= LevelFilter::DEBUG {
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
        }

        let result = match self.subcommand {
            Some(subcommand) => match subcommand {
                CliRootCommands::Install(args) => internal::install_cli_from_args(args),
                CliRootCommands::Uninstall => uninstall_command().await,
                CliRootCommands::Update { no_confirm } => {
                    installation::update_cli(no_confirm).await
                }
                CliRootCommands::Tips(tips_subcommand) => tips_subcommand.execute().await,
                CliRootCommands::Daemon => {
                    let res = daemon().await;
                    if let Err(err) = &res {
                        std::fs::write(
                            fig_directories::fig_dir()
                                .unwrap()
                                .join("logs")
                                .join("daemon-exit.log"),
                            format!("{:?}", err),
                        )
                        .ok();
                    }
                    res
                }
                CliRootCommands::Diagnostic { format, force } => {
                    diagnostics::diagnostics_cli(format, force).await
                }
                CliRootCommands::Init { shell, when } => init::shell_init_cli(&shell, &when).await,
                CliRootCommands::Source => source::source_cli().await,
                CliRootCommands::Login { refresh } => auth::login_cli(refresh).await,
                CliRootCommands::Logout => auth::logout_cli().await,
                CliRootCommands::User => auth::user_info_cli().await,
                CliRootCommands::Doctor { verbose, strict } => {
                    doctor::doctor_cli(verbose, strict).await
                }
                CliRootCommands::Invite => invite::invite_cli().await,
                CliRootCommands::Tweet => tweet::tweet_cli(),
                CliRootCommands::App(app_subcommand) => app_subcommand.execute().await,
                CliRootCommands::Hook(hook_subcommand) => {
                    // Hooks should exit silently on failure.
                    if hook_subcommand.execute().await.is_err() {
                        exit(1);
                    }
                    Ok(())
                }
                CliRootCommands::Theme { theme } => theme::theme_cli(theme).await,
                CliRootCommands::Settings(settings_args) => settings_args.execute().await,
                CliRootCommands::Debug(debug_subcommand) => debug_subcommand.execute().await,
                CliRootCommands::Issue { force, description } => {
                    issue::issue_cli(force, description).await
                }
                CliRootCommands::Completion { shell } => {
                    println!(
                        "{}",
                        match shell {
                            Shells::Bash =>
                                Cli::generation_completions(clap_complete::shells::Bash),
                            Shells::Fish =>
                                Cli::generation_completions(clap_complete::shells::Fish),
                            Shells::Zsh => Cli::generation_completions(clap_complete::shells::Zsh),
                            Shells::Fig => Cli::generation_completions(clap_complete_fig::Fig),
                        }
                    );
                    Ok(())
                }
                CliRootCommands::Internal(internal_subcommand) => {
                    internal_subcommand.execute().await
                }
                CliRootCommands::Launch => {
                    let app_res = app::launch_fig_cli();
                    if let Ok(daemon) = get_daemon() {
                        daemon.start().ok();
                    }
                    app_res
                }
                CliRootCommands::Quit => {
                    let app_res = app::quit_fig().await;
                    if let Ok(daemon) = get_daemon() {
                        daemon.stop().ok();
                    }
                    app_res
                }
                CliRootCommands::Restart { process } => match process {
                    Processes::App => {
                        get_daemon().and_then(|d| d.restart()).ok();
                        app::restart_fig().await
                    }
                    Processes::Daemon => get_daemon().and_then(|d| d.restart()),
                },
                CliRootCommands::Alpha => root_command().await,
                CliRootCommands::Onboarding => AppSubcommand::Onboarding.execute().await,
                CliRootCommands::FigAppRunning => {
                    println!("{}", if is_app_running() { "1" } else { "0" });
                    Ok(())
                }
                CliRootCommands::Plugins(plugins_subcommand) => plugins_subcommand.execute().await,
            },
            // Root command
            None => root_command().await,
        };

        if let Err(e) = result {
            if env_level > LevelFilter::INFO {
                eprintln!("{:?}", e);
            } else {
                eprintln!("{}", e);
            }
            exit(1);
        }
    }

    fn generation_completions(gen: impl clap_complete::Generator) -> String {
        let mut cli = Cli::command();
        let mut buffer = Vec::new();

        clap_complete::generate(gen, &mut cli, env!("CARGO_PKG_NAME"), &mut buffer);

        String::from_utf8_lossy(&buffer).into()
    }
}

async fn uninstall_command() -> Result<()> {
    let should_uninstall = dialoguer::Confirm::with_theme(&dialoguer_theme())
        .with_prompt("Are you sure you want to uninstall Fig?")
        .interact()?;

    if !should_uninstall {
        println!("Phew...");
        return Ok(());
    }

    let success = if launch_fig(LaunchOptions::new().wait_for_activation().verbose()).is_ok() {
        fig_ipc::command::uninstall_command().await.is_ok()
    } else {
        false
    };

    if !success {
        println!("\nFig is not running. Please launch Fig and try again to complete uninstall.\n");
    }

    Ok(())
}

async fn root_command() -> Result<()> {
    // Launch fig if it is not running

    cfg_if! {
        if #[cfg(target_os = "macos")] {
            use fig_auth::is_logged_in;
            use fig_ipc::command::{open_ui_element, quit_command};
            use fig_proto::local::UiElement;
            use std::time::Duration;

            if !is_logged_in() && is_app_running() {
                if quit_command().await.is_err() {
                    anyhow::bail!(
                        "\nFig is running but you are not logged in. Please quit Fig from the menu\
                        bar and try again\n"
                    );
                }
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }

            launch_fig(LaunchOptions::new().wait_for_activation().verbose())?;

            if is_logged_in() {
                open_ui_element(UiElement::MissionControl)
                    .await
                    .context("\nCould not launch fig\n")?;
            }
        } else {
            use crossterm::style::Stylize;

            println!(
                "\n→ Opening {}...\n",
                "https://app.fig.io".magenta().underlined()
            );
            util::open_url("https://app.fig.io").ok();
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
