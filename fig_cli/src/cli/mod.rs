//! CLI functionality

mod ai;
pub mod app;
mod autocomplete;
mod completion;
mod debug;
mod diagnostics;
mod doctor;
mod hook;
mod init;
mod installation;
mod integrations;
pub mod internal;
mod issue;
mod man;
mod settings;
mod telemetry;
mod theme;
mod tips;
// mod tweet;
mod uninstall;
mod update;
mod user;

use clap::{
    CommandFactory,
    Parser,
    Subcommand,
    ValueEnum,
};
use crossterm::style::Stylize;
use eyre::{
    Result,
    WrapErr,
};
use fig_ipc::local::open_ui_element;
use fig_log::Logger;
use fig_proto::local::UiElement;
use fig_request::auth::is_logged_in;
use fig_util::desktop::{
    launch_fig_desktop,
    LaunchArgs,
};
use fig_util::{
    manifest,
    open_url_async,
    system_info,
};
use tracing::debug;
use tracing::level_filters::LevelFilter;

use self::app::AppSubcommand;
use self::integrations::IntegrationsSubcommands;

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
    /// CodeWhisperer process
    App,
}

/// Top level cli commands
#[deny(missing_docs)]
#[derive(Debug, PartialEq, Subcommand)]
pub enum CliRootCommands {
    /// Interact with the desktop app
    #[command(subcommand)]
    App(app::AppSubcommand),
    /// Autocomplete commands
    #[command(subcommand)]
    Autocomplete(autocomplete::AutocompleteSubcommand),
    /// Hook commands
    #[command(subcommand, hide = true)]
    Hook(hook::HookSubcommand),
    /// Debug CodeWhisperer
    #[command(subcommand)]
    Debug(debug::DebugSubcommand),
    /// Customize appearance & behavior
    #[command(alias("setting"))]
    Settings(settings::SettingsArgs),
    /// Enable/disable fig tips
    #[command(subcommand)]
    Tips(tips::TipsSubcommand),
    /// Install fig cli components
    Install(internal::InstallArgs),
    /// Uninstall fig
    #[command(hide = true)]
    Uninstall {
        /// Force uninstall
        #[arg(long, short = 'y')]
        no_confirm: bool,
    },
    /// Update dotfiles
    Update {
        /// (deprecated) Use --non-interactive instead
        #[deprecated = "Use --non-interactive instead"]
        #[arg(long, hide = true)]
        no_confirm: bool,
        /// Don't prompt for confirmation
        #[arg(long, short = 'y')]
        non_interactive: bool,
        /// Relaunch into dashboard after update (false will launch in background)
        #[arg(long, default_value = "true")]
        relaunch_dashboard: bool,
        /// Uses rollout
        #[arg(long)]
        rollout: bool,
    },
    /// Run diagnostic tests
    #[command(alias("diagnostics"))]
    Diagnostic(diagnostics::DiagnosticArgs),
    /// Generate the dotfiles for the given shell
    Init(init::InitArgs),
    /// Get or set theme
    Theme(theme::ThemeArgs),
    // Tweet about CodeWhisperer
    // Tweet,
    /// Create a new Github issue
    Issue(issue::IssueArgs),
    /// Root level user subcommands
    #[command(flatten)]
    RootUser(user::RootUserSubcommand),
    /// Manage your fig user
    #[command(subcommand)]
    User(user::UserSubcommand),
    /// Check CodeWhisperer is properly configured
    Doctor(doctor::DoctorArgs),
    /// Generate the completion spec for CodeWhisperer
    #[command(hide = true)]
    Completion(completion::CompletionArgs),
    /// Internal subcommands used for CodeWhisperer
    #[command(subcommand, hide = true)]
    Internal(internal::InternalSubcommand),
    /// Launch the CodeWhisperer desktop app
    Launch,
    /// Quit the CodeWhisperer desktop app
    Quit,
    /// Restart the CodeWhisperer desktop app
    Restart {
        /// The process to restart
        #[arg(value_enum, default_value_t = Processes::App, hide = true)]
        process: Processes,
    },
    /// Run the CodeWhisperer tutorial
    #[command(hide = true)]
    Onboarding,
    /// Open manual page
    Man(man::ManArgs),
    /// Manage system integrations
    #[command(subcommand, alias("integration"))]
    Integrations(IntegrationsSubcommands),
    /// English -> Bash translation
    Ai(ai::AiArgs),
    /// Enable/disable telemetry
    #[command(subcommand, hide = true)]
    Telemetry(telemetry::TelemetrySubcommand),
    /// Version
    Version,
    /// Print help for all subcommands
    HelpAll,
    /// Open the fig dashboard
    Dashboard,
}

#[derive(Debug, Parser)]
#[command(version, about)]
#[command(help_template = "\x1B[1;95m
 cw\x1B[0m (Amazon CodeWhisperer CLI)
╭──────────────────────────────────────────────────────╮
│ \x1B[1mcw\x1B[0m          \x1B[0;90mOpen the CodeWhisperer Dashboard\x1B[0m         │ 
│ \x1B[1mcw doctor\x1B[0m   \x1B[0;90mDebug CodeWhisperer installation issues\x1B[0m  │ 
╰──────────────────────────────────────────────────────╯

 \x1B[1;95mPopular Subcommands\x1B[0m           \x1B[1;90mUsage:\x1B[0;90m cw [subcommand]\x1B[0m
╭──────────────────────────────────────────────────────╮
│ \x1B[1mai\x1B[0m             \x1B[0;90mAi :D\x1B[0m                                 |
│ \x1B[1msettings\x1B[0m       \x1B[0;90mCustomize appearance & behavior\x1B[0m       │
│ \x1B[1mquit\x1B[0m           \x1B[0;90mQuit the CodeWhisperer app\x1B[0m            │
╰──────────────────────────────────────────────────────╯

 \x1B[0;90mTo see all subcommands, use:\x1B[0m
  > cw help-all
ㅤ
")]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Option<CliRootCommands>,
}

impl Cli {
    pub async fn execute(self) -> Result<()> {
        let mut logger = Logger::new();
        // All other cli commands print logs to ~/.fig/logs/cli.log
        if std::env::var_os("FIG_LOG_STDOUT").is_some() {
            logger = logger.with_file("cli.log").with_max_file_size(10_000_000).with_stdout();
        } else if fig_log::get_max_fig_log_level() >= LevelFilter::DEBUG {
            logger = logger.with_file("cli.log").with_max_file_size(10_000_000);
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
                CliRootCommands::Autocomplete(args) => args.execute().await,
                CliRootCommands::Uninstall { no_confirm } => uninstall::uninstall_command(no_confirm).await,
                CliRootCommands::Update {
                    non_interactive,
                    relaunch_dashboard,
                    rollout,
                    ..
                } => update::update(non_interactive, relaunch_dashboard, rollout).await,
                CliRootCommands::Tips(tips_subcommand) => tips_subcommand.execute().await,
                CliRootCommands::Diagnostic(args) => args.execute().await,
                CliRootCommands::Init(args) => args.execute().await,
                CliRootCommands::User(user) => user.execute().await,
                CliRootCommands::RootUser(root_user) => root_user.execute().await,
                CliRootCommands::Doctor(args) => args.execute().await,
                // CliRootCommands::Tweet => tweet::tweet_cli(),
                CliRootCommands::App(app_subcommand) => app_subcommand.execute().await,
                CliRootCommands::Hook(hook_subcommand) => hook_subcommand.execute().await,
                CliRootCommands::Theme(theme_args) => theme_args.execute().await,
                CliRootCommands::Settings(settings_args) => settings_args.execute().await,
                CliRootCommands::Debug(debug_subcommand) => debug_subcommand.execute().await,
                CliRootCommands::Issue(args) => args.execute().await,
                CliRootCommands::Completion(args) => args.execute(),
                CliRootCommands::Internal(internal_subcommand) => internal_subcommand.execute().await,
                CliRootCommands::Launch => launch_dashboard().await,
                CliRootCommands::Quit => crate::util::quit_fig(true).await,
                CliRootCommands::Restart { process } => match process {
                    Processes::App => {
                        app::restart_fig().await?;
                        launch_dashboard().await
                    },
                    Processes::Daemon => Ok(()),
                },
                CliRootCommands::Onboarding => AppSubcommand::Onboarding.execute().await,
                CliRootCommands::Man(args) => args.execute(),
                CliRootCommands::Integrations(subcommand) => subcommand.execute().await,
                CliRootCommands::Ai(args) => args.execute().await,
                CliRootCommands::Telemetry(subcommand) => subcommand.execute().await,
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
                CliRootCommands::Dashboard => launch_dashboard().await,
            },
            // Root command
            None => launch_dashboard().await,
        }
    }
}

async fn launch_dashboard() -> Result<()> {
    if manifest::is_headless() || system_info::is_remote() {
        match open_url_async("https://app.fig.io").await {
            Ok(_) => eprintln!("Opening {} in browser...", "https://app.fig.io".magenta()),
            Err(_) => eprintln!("Go to {} to open the dashboard", "https://app.fig.io".magenta()),
        }
        return Ok(());
    }

    launch_fig_desktop(LaunchArgs {
        wait_for_socket: true,
        open_dashboard: true,
        immediate_update: true,
        verbose: true,
    })?;

    let route = match is_logged_in() {
        true => Some("/".into()),
        false => None,
    };

    open_ui_element(UiElement::MissionControl, route)
        .await
        .context("Failed to open dashboard")?;

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
            Cli::parse_from(["codewhisperer", "restart", "app"]).subcommand,
            Some(CliRootCommands::Restart {
                process: Processes::App
            })
        );

        assert_eq!(
            Cli::parse_from(["codewhisperer", "restart", "daemon"]).subcommand,
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
                "cw",
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

    #[test]
    fn test_ghost_text() {
        use internal::InternalSubcommand;

        assert_eq!(
            Cli::parse_from(["cw", "_", "ghost-text", "--buffer", ""]).subcommand,
            Some(CliRootCommands::Internal(InternalSubcommand::GhostText {
                buffer: "".to_string()
            }))
        );

        assert_eq!(
            Cli::parse_from(["cw", "_", "ghost-text", "--buffer", "foo"]).subcommand,
            Some(CliRootCommands::Internal(InternalSubcommand::GhostText {
                buffer: "foo".to_string()
            }))
        );

        assert_eq!(
            Cli::parse_from(["cw", "_", "ghost-text", "--buffer", "-"]).subcommand,
            Some(CliRootCommands::Internal(InternalSubcommand::GhostText {
                buffer: "-".to_string()
            }))
        );

        assert_eq!(
            Cli::parse_from(["cw", "_", "ghost-text", "--buffer", "--"]).subcommand,
            Some(CliRootCommands::Internal(InternalSubcommand::GhostText {
                buffer: "--".to_string()
            }))
        );

        assert_eq!(
            Cli::parse_from(["cw", "_", "ghost-text", "--buffer", "--foo bar"]).subcommand,
            Some(CliRootCommands::Internal(InternalSubcommand::GhostText {
                buffer: "--foo bar".to_string()
            }))
        );

        assert_eq!(
            Cli::parse_from(["cw", "_", "ghost-text-accept", "--buffer", "abc", "--suggestion", "def"]).subcommand,
            Some(CliRootCommands::Internal(InternalSubcommand::GhostTextAccept {
                buffer: "abc".to_string(),
                suggestion: "def".to_string()
            }))
        );
    }
}
