//! CLI functionality

mod ai;
pub mod app;
mod completion;
mod debug;
mod diagnostics;
mod doctor;
mod hook;
mod init;
mod installation;
mod integrations;
mod internal;
mod invite;
mod issue;
mod man;
mod plugins;
mod pro;
mod settings;
mod source;
mod ssh;
mod team;
mod theme;
mod tips;
mod tweet;
mod uninstall;
mod user;
mod workflow;

use cfg_if::cfg_if;
use clap::{
    IntoApp,
    Parser,
    Subcommand,
    ValueEnum,
};
use color_eyre::owo_colors::OwoColorize;
use eyre::{
    Result,
    WrapErr,
};
use fig_log::Logger;
use fig_util::directories;
use tracing::debug;
use tracing::level_filters::LevelFilter;

use self::app::AppSubcommand;
use self::integrations::IntegrationsSubcommands;
use self::plugins::PluginsSubcommands;
use crate::daemon::{
    daemon,
    get_daemon,
};
use crate::util::{
    dialoguer_theme,
    is_app_running,
    launch_fig,
    LaunchArgs,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// Outputs the results as markdown
    #[default]
    Plain,
    /// Outputs the results as JSON
    Json,
    /// Outputs the results as pretty print JSON
    JsonPretty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Processes {
    /// Daemon process
    Daemon,
    /// Fig process
    App,
}

/// Top level cli commands
#[deny(missing_docs)]
#[derive(Debug, Subcommand)]
pub enum CliRootCommands {
    /// Interact with the desktop app
    #[clap(subcommand)]
    App(app::AppSubcommand),
    /// Hook commands
    #[clap(subcommand, hide = true)]
    Hook(hook::HookSubcommand),
    /// Debug Fig
    #[clap(subcommand)]
    Debug(debug::DebugSubcommand),
    /// Customize appearance & behavior
    Settings(settings::SettingsArgs),
    /// Enable/disable fig tips
    #[clap(subcommand)]
    Tips(tips::TipsSubcommand),
    /// Install fig cli components
    Install(internal::InstallArgs),
    /// Enable/disable fig SSH integration
    Ssh(ssh::SshSubcommand),
    /// Uninstall fig
    #[clap(hide = true)]
    Uninstall,
    /// Update dotfiles
    Update {
        /// Force update
        #[clap(long, short = 'y', value_parser)]
        no_confirm: bool,
    },
    /// Run the daemon
    #[clap(hide = true)]
    Daemon,
    /// Run diagnostic tests
    #[clap(alias("diagnostic"))]
    Diagnostic(diagnostics::DiagnosticArgs),
    /// Generate the dotfiles for the given shell
    Init(init::InitArgs),
    /// Sync your latest dotfiles
    Source,
    /// Get or set theme
    Theme(theme::ThemeArgs),
    /// Invite friends to Fig
    Invite,
    /// Tweet about Fig
    Tweet,
    /// Create a new Github issue
    Issue(issue::IssueArgs),
    /// Root level user subcommands
    #[clap(flatten)]
    RootUser(user::RootUserSubcommand),
    /// Manage your fig user
    #[clap(subcommand)]
    User(user::UserSubcommand),
    /// Manage your fig team
    Team(team::TeamCommand),
    /// Check Fig is properly configured
    Doctor(doctor::DoctorArgs),
    /// Generate the completion spec for Fig
    #[clap(hide = true)]
    Completion(completion::CompletionArgs),
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
        #[clap(value_enum, value_parser, default_value_t = Processes::App, hide = true)]
        process: Processes,
    },
    /// Run the Fig tutorial
    #[clap(hide = true)]
    Onboarding,
    /// Manage your shell plugins with Fig
    #[clap(subcommand)]
    Plugins(PluginsSubcommands),
    /// Open manual page
    Man(man::ManArgs),
    /// Fig Workflows
    #[clap(aliases(&["run", "r", "workflows", "snippet", "snippets", "flow", "flows"]))]
    Workflow(workflow::WorkflowArgs),
    /// Manage system integrations
    #[clap(subcommand, alias("integration"))]
    Integrations(IntegrationsSubcommands),
    /// English -> Bash translation
    Ai(ai::AiArgs),
    /// Fig Pro
    Pro,
    /// Version
    Version,
    /// Print help for all subcommands
    HelpAll,

    /// (LEGACY) Old hook that was being used somewhere
    #[clap(name = "app:running", hide = true)]
    LegacyAppRunning,
    /// (LEGACY) Old ssh hook that might be in ~/.ssh/config
    #[clap(name = "bg:ssh", hide = true)]
    LegacyBgSsh,
    /// (LEGACY) Old tmux hook that might be in ~/.tmux.conf
    #[clap(name = "bg:tmux", hide = true)]
    LegacyBgTmux {
        /// Tmux args
        #[clap(value_parser)]
        args: Vec<String>,
    },
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
│ \x1B[1mquit\x1B[0m           \x1B[0;90mQuit the Fig app\x1B[0m                  │
│ \x1B[1muninstall\x1B[0m      \x1B[0;90mUninstall Fig\x1B[0m                     │
╰──────────────────────────────────────────────────╯

 \x1B[0;90mFor more info on a specific command, use:\x1B[0m
  > fig help [command]

 Run \x1B[1;95mfig\x1B[0m to get started
")]
pub struct Cli {
    #[clap(subcommand)]
    pub subcommand: Option<CliRootCommands>,
}

impl Cli {
    pub async fn execute(self) -> Result<()> {
        let mut logger = Logger::new();
        match self.subcommand {
            Some(CliRootCommands::Daemon) => {
                // Remove the daemon log file if it is >10Mb
                let daemon_log_file = fig_util::directories::fig_dir()?.join("logs").join("daemon.log");
                if daemon_log_file.exists() {
                    let metadata = std::fs::metadata(&daemon_log_file)?;
                    if metadata.len() > 10_000_000 {
                        std::fs::remove_file(&daemon_log_file)?;
                    }
                }

                if fig_settings::state::get_bool_or("logging.daemon", false) {
                    // The daemon prints all logs to stdout
                    logger = logger.with_stdout();
                }
            },
            _ => {
                // All other cli commands print logs to ~/.fig/logs/cli.log
                if std::env::var_os("FIG_LOG_STDOUT").is_some() {
                    logger = logger.with_file("cli.log").with_max_file_size(10_000_000).with_stdout();
                } else if *fig_log::FIG_LOG_LEVEL >= LevelFilter::DEBUG {
                    logger = logger.with_file("cli.log").with_max_file_size(10_000_000);
                }
            },
        }

        let _logger_guard = logger.init().expect("Failed to init logger");
        debug!("Command ran: {:?}", std::env::args().collect::<Vec<_>>());

        match self.subcommand {
            Some(subcommand) => match subcommand {
                CliRootCommands::Install(args) => {
                    if let internal::InstallArgs { input_method: true, .. } = args {
                        cfg_if::cfg_if! {
                            if #[cfg(target_os = "macos")] {
                                use fig_ipc::local::open_ui_element;
                                use fig_proto::local::UiElement;

                                open_ui_element(UiElement::InputMethodPrompt, None)
                                    .await
                                    .context("\nCould not launch fig\n")?;
                            } else {
                                Err(eyre::eyre!("input method is only implemented on macOS"))?;
                            }
                        }

                        Ok(())
                    } else {
                        internal::install_cli_from_args(args)
                    }
                },
                CliRootCommands::Uninstall => uninstall::uninstall_command().await,
                CliRootCommands::Update { no_confirm } => installation::update(no_confirm).await.map(|_| ()),
                CliRootCommands::Ssh(ssh_subcommand) => ssh_subcommand.execute().await,
                CliRootCommands::Tips(tips_subcommand) => tips_subcommand.execute().await,
                CliRootCommands::Daemon => {
                    let res = daemon().await;
                    if let Err(err) = &res {
                        std::fs::write(
                            directories::fig_dir().unwrap().join("logs").join("daemon-exit.log"),
                            format!("{:?}", err),
                        )
                        .ok();
                    }
                    res
                },
                CliRootCommands::Diagnostic(args) => args.execute().await,
                CliRootCommands::Init(args) => args.execute().await,
                CliRootCommands::Source => source::source_cli().await,
                CliRootCommands::User(user) => user.execute().await,
                CliRootCommands::RootUser(root_user) => root_user.execute().await,
                CliRootCommands::Team(team) => team.execute().await,
                CliRootCommands::Doctor(args) => args.execute().await,
                CliRootCommands::Invite => invite::invite_cli().await,
                CliRootCommands::Tweet => tweet::tweet_cli(),
                CliRootCommands::App(app_subcommand) => app_subcommand.execute().await,
                CliRootCommands::Hook(hook_subcommand) => hook_subcommand.execute().await,
                CliRootCommands::Theme(theme_args) => theme_args.execute().await,
                CliRootCommands::Settings(settings_args) => settings_args.execute().await,
                CliRootCommands::Debug(debug_subcommand) => debug_subcommand.execute().await,
                CliRootCommands::Issue(args) => args.execute().await,
                CliRootCommands::Completion(args) => args.execute(),
                CliRootCommands::Internal(internal_subcommand) => internal_subcommand.execute().await,
                CliRootCommands::Launch => launch_fig(LaunchArgs {
                    print_running: true,
                    print_launching: true,
                    wait_for_launch: true,
                }),
                CliRootCommands::Quit => app::quit_fig().await,
                CliRootCommands::Restart { process } => match process {
                    Processes::App => app::restart_fig().await,
                    Processes::Daemon => get_daemon().and_then(|d| d.restart()),
                },
                CliRootCommands::Onboarding => AppSubcommand::Onboarding.execute().await,
                CliRootCommands::Plugins(plugins_subcommand) => plugins_subcommand.execute().await,
                CliRootCommands::Man(args) => args.execute(),
                CliRootCommands::Workflow(args) => args.execute().await,
                CliRootCommands::Integrations(subcommand) => subcommand.execute().await,
                CliRootCommands::Ai(args) => args.execute().await,
                CliRootCommands::Pro => pro::execute().await,
                CliRootCommands::Version => {
                    print!("{}", Self::command().render_version());
                    Ok(())
                },
                CliRootCommands::HelpAll => {
                    let mut cmd = Self::command().help_template("{all-args}");
                    eprintln!();
                    eprintln!(
                        "  \x1B[1m███████╗██╗ ██████╗
  ██╔════╝██║██╔════╝
  █████╗  ██║██║  ███╗
  ██╔══╝  ██║██║   ██║
  ██║     ██║╚██████╔╝
  ╚═╝     ╚═╝ ╚═════╝ CLI\x1B[0m\n"
                    );
                    println!("{}\n    {}\n", "USAGE:".yellow(), "fig [OPTIONS] [SUBCOMMAND]".green());
                    cmd.print_long_help()?;
                    Ok(())
                },

                CliRootCommands::LegacyAppRunning => {
                    println!("{}", if is_app_running() { "1" } else { "0" });
                    Ok(())
                },
                CliRootCommands::LegacyBgSsh => Ok(()),
                CliRootCommands::LegacyBgTmux { .. } => Ok(()),
            },
            // Root command
            None => root_command().await,
        }
    }
}

async fn root_command() -> Result<()> {
    // Launch fig if it is not running
    cfg_if! {
        if #[cfg(target_os = "macos")] {
            use fig_auth::is_logged_in;
            use fig_ipc::local::{open_ui_element, quit_command};
            use fig_proto::local::UiElement;
            use std::time::Duration;

            if !is_logged_in() && is_app_running() {
                if quit_command().await.is_err() {
                    eyre::bail!(
                        "Fig is running but you are not logged in. Please quit Fig from the menu\
                        bar and try again"
                    );
                }
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }

            launch_fig(LaunchArgs {
                print_running: false,
                print_launching: true,
                wait_for_launch: true,
            })?;

            if is_logged_in() {
                open_ui_element(UiElement::MissionControl, None)
                    .await
                    .context("Could not launch fig")?;
            }
        } else {
            use crossterm::style::Stylize;
            use fig_ipc::local::open_ui_element;
            use fig_proto::local::UiElement;
            use std::io::{
                stdout,
                Write,
            };

            match launch_fig(LaunchArgs { print_running: false, print_launching: true, wait_for_launch: true }) {
                Ok(()) => {
                    open_ui_element(UiElement::MissionControl, None)
                        .await
                        .context("Could not launch fig")?;
                }
                Err(_) => {
                    writeln!(
                        stdout(),
                        "Opening {}",
                        "https://app.fig.io".magenta().underlined()
                    ).ok();
                    fig_util::open_url("https://app.fig.io")?;
                }
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
