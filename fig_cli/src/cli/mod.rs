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
mod update;
mod user;
mod workflow;

use cfg_if::cfg_if;
use clap::{
    CommandFactory,
    Parser,
    Subcommand,
    ValueEnum,
};
use color_eyre::owo_colors::OwoColorize;
use eyre::{
    Result,
    WrapErr,
};
use fig_daemon::Daemon;
use fig_log::Logger;
use fig_util::{
    directories,
    is_fig_desktop_running,
    launch_fig_desktop,
};
use tracing::debug;
use tracing::level_filters::LevelFilter;

use self::app::AppSubcommand;
use self::integrations::IntegrationsSubcommands;
use self::plugins::PluginsSubcommands;
use crate::daemon::daemon;

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
#[derive(Debug, PartialEq, Subcommand)]
pub enum CliRootCommands {
    /// Interact with the desktop app
    #[command(subcommand)]
    App(app::AppSubcommand),
    /// Hook commands
    #[command(subcommand, hide = true)]
    Hook(hook::HookSubcommand),
    /// Debug Fig
    #[command(subcommand)]
    Debug(debug::DebugSubcommand),
    /// Customize appearance & behavior
    Settings(settings::SettingsArgs),
    /// Enable/disable fig tips
    #[command(subcommand)]
    Tips(tips::TipsSubcommand),
    /// Install fig cli components
    Install(internal::InstallArgs),
    /// Enable/disable fig SSH integration
    Ssh(ssh::SshSubcommand),
    /// Uninstall fig
    #[command(hide = true)]
    Uninstall {
        /// Force uninstall
        #[arg(long, short = 'y')]
        no_confirm: bool,
    },
    /// Update dotfiles
    Update {
        /// Force update
        #[arg(long, short = 'y')]
        no_confirm: bool,
    },
    /// Run the daemon
    #[command(hide = true)]
    Daemon,
    /// Run diagnostic tests
    #[command(alias("diagnostics"))]
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
    #[command(flatten)]
    RootUser(user::RootUserSubcommand),
    /// Manage your fig user
    #[command(subcommand)]
    User(user::UserSubcommand),
    /// Manage your fig team
    Team(team::TeamCommand),
    /// Check Fig is properly configured
    Doctor(doctor::DoctorArgs),
    /// Generate the completion spec for Fig
    #[command(hide = true)]
    Completion(completion::CompletionArgs),
    /// Internal subcommands used for Fig
    #[command(subcommand, hide = true)]
    Internal(internal::InternalSubcommand),
    /// Launch the Fig desktop app
    Launch,
    /// Quit the Fig desktop app
    Quit,
    /// Restart the Fig desktop app
    Restart {
        /// The process to restart
        #[arg(value_enum, default_value_t = Processes::App, hide = true)]
        process: Processes,
    },
    /// Run the Fig tutorial
    #[command(hide = true)]
    Onboarding,
    /// Manage your shell plugins with Fig
    #[command(subcommand)]
    Plugins(PluginsSubcommands),
    /// Open manual page
    Man(man::ManArgs),
    /// Fig Workflows
    #[command(aliases(&["run", "r", "workflows", "snippet", "snippets", "flow", "flows"]))]
    Workflow(workflow::WorkflowArgs),
    /// Manage system integrations
    #[command(subcommand, alias("integration"))]
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
    #[command(name = "app:running", hide = true)]
    LegacyAppRunning,
    /// (LEGACY) Old ssh hook that might be in ~/.ssh/config
    #[command(name = "bg:ssh", hide = true)]
    LegacyBgSsh,
    /// (LEGACY) Old tmux hook that might be in ~/.tmux.conf
    #[command(name = "bg:tmux", hide = true)]
    LegacyBgTmux {
        /// Tmux args
        args: Vec<String>,
    },
}

#[derive(Debug, Parser)]
#[command(version, about)]
#[command(help_template = "
ㅤ\x1B[1m███████╗██╗ ██████╗
  ██╔════╝██║██╔════╝
  █████╗  ██║██║  ███╗
  ██╔══╝  ██║██║   ██║
  ██║     ██║╚██████╔╝
  ╚═╝     ╚═╝ ╚═════╝ CLI\x1B[0m

╭────────────────────────────────────────────────────╮
│ \x1B[1mfig\x1B[0m            \x1B[0;90mOpen the Fig Dashboard\x1B[0m              │ 
│ \x1B[1mfig doctor\x1B[0m     \x1B[0;90mDebug Fig installation issues\x1B[0m       │ 
╰────────────────────────────────────────────────────╯

 \x1B[1;95mPopular Subcommands\x1B[0m           \x1B[1;90mUsage:\x1B[0;90m fig [subcommand]\x1B[0m
╭────────────────────────────────────────────────────╮
│ \x1B[1mai\x1B[0m             \x1B[0;90mTranslate English → Bash\x1B[0m            │
│ \x1B[1msettings\x1B[0m       \x1B[0;90mCustomize appearance & behavior\x1B[0m     │
│ \x1B[1mtweet\x1B[0m          \x1B[0;90mTweet about Fig\x1B[0m                     │
│ \x1B[1mupdate\x1B[0m         \x1B[0;90mCheck for updates\x1B[0m                   │
│ \x1B[1missue\x1B[0m          \x1B[0;90mCreate a new GitHub issue\x1B[0m           │
│ \x1B[1mquit\x1B[0m           \x1B[0;90mQuit the Fig app\x1B[0m                    │
╰────────────────────────────────────────────────────╯

 \x1B[0;90mTo see all subcommands, use:\x1B[0m
  > fig help-all
ㅤ
")]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Option<CliRootCommands>,
}

impl Cli {
    pub async fn execute(self) -> Result<()> {
        let mut logger = Logger::new();
        match self.subcommand {
            Some(CliRootCommands::Daemon) => {
                // Remove the daemon log file if it is >10Mb
                let daemon_log_file = fig_util::directories::fig_dir()?.join("logs").join("");
                if daemon_log_file.exists() {
                    let metadata = std::fs::metadata(&daemon_log_file)?;
                    if metadata.len() > 10_000_000 {
                        std::fs::remove_file(&daemon_log_file)?;
                    }
                }

                logger = logger.with_file("daemon.log").with_stdout();
            },
            _ => {
                // All other cli commands print logs to ~/.fig/logs/cli.log
                if std::env::var_os("FIG_LOG_STDOUT").is_some() {
                    logger = logger.with_file("cli.log").with_max_file_size(10_000_000).with_stdout();
                } else if fig_log::get_max_fig_log_level() >= LevelFilter::DEBUG {
                    logger = logger.with_file("cli.log").with_max_file_size(10_000_000);
                }
            },
        }

        let _logger_guard = logger.init().expect("Failed to init logger");
        debug!("Command ran: {:?}", std::env::args().collect::<Vec<_>>());

        match self.subcommand {
            Some(subcommand) => match subcommand {
                CliRootCommands::Install(args) => {
                    let no_confirm = args.no_confirm;
                    let force = args.force;
                    installation::install_cli(args.into(), no_confirm, force).await
                },
                CliRootCommands::Uninstall { no_confirm } => uninstall::uninstall_command(no_confirm).await,
                CliRootCommands::Update { no_confirm } => update::update(no_confirm).await,
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
                CliRootCommands::Launch => {
                    if is_fig_desktop_running() {
                        println!("Fig is already running!");
                        return Ok(());
                    }
                    launch_fig_desktop(true, true)?;
                    Ok(())
                },
                CliRootCommands::Quit => crate::util::quit_fig(true).await,
                CliRootCommands::Restart { process } => match process {
                    Processes::App => app::restart_fig().await,
                    Processes::Daemon => Daemon::default().restart().await.context("Failed to restart daemon"),
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
                    println!("{}", if is_fig_desktop_running() { "1" } else { "0" });
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
            use fig_request::auth::is_logged_in;
            use fig_ipc::local::{open_ui_element, quit_command};
            use fig_proto::local::UiElement;
            use std::time::Duration;

            if !is_logged_in() && is_fig_desktop_running() {
                if quit_command().await.is_err() {
                    eyre::bail!(
                        "Fig is running but you are not logged in. Please quit Fig from the menu\
                        bar and try again"
                    );
                }
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }

            launch_fig_desktop(true, true)?;

            if is_logged_in() {
                open_ui_element(UiElement::MissionControl, None)
                    .await
                    .context("Could not launch fig")?;
            }
        } else {
            use fig_ipc::local::open_ui_element;
            use fig_proto::local::UiElement;

            if fig_util::manifest::is_headless() {
                eyre::bail!("Launching Fig from headless installs is not yet supported");
            }

            launch_fig_desktop(true, true)?;
            open_ui_element(UiElement::MissionControl, None).await.context("Failed to open Fig")?;
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

    /// This test validates that the restart command maintains the same CLI facing definition
    ///
    /// If this changes, you must also change how it is called from within fig_install
    /// and (possibly) other locations as well
    #[test]
    fn test_restart() {
        assert_eq!(
            Cli::parse_from(["fig", "restart", "app"]).subcommand,
            Some(CliRootCommands::Restart {
                process: Processes::App
            })
        );

        assert_eq!(
            Cli::parse_from(["fig", "restart", "daemon"]).subcommand,
            Some(CliRootCommands::Restart {
                process: Processes::Daemon
            })
        );
    }

    /// This test validates that the internal input method installation command maintains the same
    /// CLI facing definition
    ///
    /// If this changes, you must also change how it is called from within
    /// fig_integrations::input_method
    #[cfg(target_os = "macos")]
    #[test]
    fn test_input_method_installation() {
        use internal::InternalSubcommand;
        assert_eq!(
            Cli::parse_from([
                "fig",
                "_",
                "attempt-to-finish-input-method-installation",
                "/path/to/bundle.app"
            ])
            .subcommand,
            Some(CliRootCommands::Internal(
                InternalSubcommand::AttemptToFinishInputMethodInstallation {
                    bundle_path: Some(std::path::PathBuf::from("/path/to/bundle.app"))
                }
            ))
        );
    }
}
