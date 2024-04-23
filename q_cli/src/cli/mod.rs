//! CLI functionality

pub mod app;
mod chat;
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
mod settings;
mod telemetry;
mod theme;
mod translate;
mod uninstall;
mod update;
mod user;

use auth::is_logged_in;
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
use fig_util::desktop::{
    launch_fig_desktop,
    LaunchArgs,
};
use fig_util::{
    manifest,
    system_info,
    CLI_BINARY_NAME,
};
use serde::Serialize;
use tracing::debug;
use tracing::level_filters::LevelFilter;

use self::app::AppSubcommand;
use self::integrations::IntegrationsSubcommands;
use self::user::RootUserSubcommand;

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

impl OutputFormat {
    pub fn print<T, TFn, J, JFn>(&self, text_fn: TFn, json_fn: JFn)
    where
        T: std::fmt::Display,
        TFn: FnOnce() -> T,
        J: Serialize,
        JFn: FnOnce() -> J,
    {
        match self {
            OutputFormat::Plain => println!("{}", text_fn()),
            OutputFormat::Json => println!("{}", serde_json::to_string(&json_fn()).unwrap()),
            OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&json_fn()).unwrap()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Processes {
    /// Daemon process
    Daemon,
    /// Desktop Process
    App,
}

/// Top level cli commands
#[deny(missing_docs)]
#[derive(Debug, PartialEq, Subcommand)]
pub enum CliRootCommands {
    /// Interact with the desktop app
    #[command(subcommand, hide = true)]
    App(app::AppSubcommand),
    /// Hook commands
    #[command(subcommand, hide = true)]
    Hook(hook::HookSubcommand),
    /// Debug the app
    #[command(subcommand)]
    Debug(debug::DebugSubcommand),
    /// Customize appearance & behavior
    #[command(alias("setting"))]
    Settings(settings::SettingsArgs),
    /// Install cli components
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
    /// Create a new Github issue
    Issue(issue::IssueArgs),
    /// Root level user subcommands
    #[command(flatten)]
    RootUser(user::RootUserSubcommand),
    /// Manage your account
    #[command(subcommand)]
    User(user::UserSubcommand),
    /// Fix and diagnose common issues
    Doctor(doctor::DoctorArgs),
    /// Generate CLI completion spec
    #[command(hide = true)]
    Completion(completion::CompletionArgs),
    /// Internal subcommands
    #[command(subcommand, hide = true)]
    Internal(internal::InternalSubcommand),
    /// Launch the desktop app
    Launch,
    /// Quit the desktop app
    Quit,
    /// Restart the desktop app
    Restart {
        /// The process to restart
        #[arg(value_enum, default_value_t = Processes::App, hide = true)]
        process: Processes,
    },
    /// Run the tutorial
    #[command(hide = true)]
    Onboarding,
    /// Manage system integrations
    #[command(subcommand, alias("integration"))]
    Integrations(IntegrationsSubcommands),
    /// Natural Language to Shell translation
    #[command(alias("ai"))]
    Translate(translate::TranslateArgs),
    /// Enable/disable telemetry
    #[command(subcommand, hide = true)]
    Telemetry(telemetry::TelemetrySubcommand),
    /// Version
    #[command(hide = true)]
    Version,
    /// Print help for all subcommands
    HelpAll,
    /// Open the dashboard
    Dashboard,
    /// AI assistant in your terminal
    #[command(alias("q"))]
    Chat {
        /// The first question to ask
        input: Option<String>,
    },
}

impl CliRootCommands {
    fn name(&self) -> &'static str {
        match self {
            CliRootCommands::App(_) => "app",
            CliRootCommands::Hook(_) => "hook",
            CliRootCommands::Debug(_) => "debug",
            CliRootCommands::Settings(_) => "settings",
            CliRootCommands::Install(_) => "install",
            CliRootCommands::Uninstall { .. } => "uninstall",
            CliRootCommands::Update { .. } => "update",
            CliRootCommands::Diagnostic(_) => "diagnostics",
            CliRootCommands::Init(_) => "init",
            CliRootCommands::Theme(_) => "theme",
            CliRootCommands::Issue(_) => "issue",
            CliRootCommands::RootUser(RootUserSubcommand::Login) => "login",
            CliRootCommands::RootUser(RootUserSubcommand::Logout) => "logout",
            CliRootCommands::RootUser(RootUserSubcommand::Whoami { .. }) => "whoami",
            CliRootCommands::User(_) => "user",
            CliRootCommands::Doctor(_) => "doctor",
            CliRootCommands::Completion(_) => "completion",
            CliRootCommands::Internal(_) => "internal",
            CliRootCommands::Launch => "launch",
            CliRootCommands::Quit => "quit",
            CliRootCommands::Restart { .. } => "restart",
            CliRootCommands::Onboarding => "onboarding",
            CliRootCommands::Integrations(_) => "integrations",
            CliRootCommands::Translate(_) => "translate",
            CliRootCommands::Telemetry(_) => "telemetry",
            CliRootCommands::Version => "version",
            CliRootCommands::HelpAll => "help-all",
            CliRootCommands::Dashboard => "dashboard",
            CliRootCommands::Chat { .. } => "chat",
        }
    }
}

#[derive(Debug, Parser)]
#[command(version, about, name = CLI_BINARY_NAME)]
#[command(help_template = "\x1B[1;95m
 q\x1B[0m (Amazon Q CLI)

 \x1B[1;95mPopular Subcommands\x1B[0m                \x1B[1;90mUsage:\x1B[0;90m q [subcommand]\x1B[0m
╭────────────────────────────────────────────────────────╮
│ \x1B[1mchat\x1B[0m             \x1B[0;90mChat with Amazon Q\x1B[0m                   │
│ \x1B[1mtranslate\x1B[0m        \x1B[0;90mNatural Language to Shell translation\x1B[0m │
│ \x1B[1mdoctor\x1B[0m           \x1B[0;90mDebug installation issues\x1B[0m             │ 
│ \x1B[1msettings\x1B[0m         \x1B[0;90mCustomize appearance & behavior\x1B[0m       │
│ \x1B[1mquit\x1B[0m             \x1B[0;90mQuit the app\x1B[0m                          │
╰────────────────────────────────────────────────────────╯

 \x1B[0;90mTo see all subcommands, use:\x1B[0m
  > q help-all

")]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Option<CliRootCommands>,
}

impl Cli {
    pub async fn execute(self) -> Result<()> {
        let mut logger = Logger::new();
        // All other cli commands print logs to ~/.fig/logs/cli.log
        if std::env::var_os("Q_LOG_STDOUT").is_some() {
            logger = logger.with_file("cli.log").with_max_file_size(10_000_000).with_stdout();
        } else if fig_log::get_max_fig_log_level() >= LevelFilter::DEBUG {
            logger = logger.with_file("cli.log").with_max_file_size(10_000_000);
        }

        let _logger_guard = logger.init().expect("Failed to init logger");
        debug!(command =? std::env::args().collect::<Vec<_>>(), "Command ran");

        self.send_telemetry().await;

        match self.subcommand {
            Some(subcommand) => match subcommand {
                CliRootCommands::Install(args) => {
                    let no_confirm = args.no_confirm;
                    let force = args.force;
                    installation::install_cli(args.into(), no_confirm, force).await
                },
                CliRootCommands::Uninstall { no_confirm } => uninstall::uninstall_command(no_confirm).await,
                CliRootCommands::Update {
                    non_interactive,
                    relaunch_dashboard,
                    rollout,
                    ..
                } => update::update(non_interactive, relaunch_dashboard, rollout).await,
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
                CliRootCommands::Integrations(subcommand) => subcommand.execute().await,
                CliRootCommands::Translate(args) => args.execute().await,
                CliRootCommands::Telemetry(subcommand) => subcommand.execute().await,
                CliRootCommands::Version => {
                    print!("{}", Self::command().render_version());
                    Ok(())
                },
                CliRootCommands::HelpAll => {
                    let mut cmd = Self::command().help_template("{all-args}");
                    eprintln!();
                    // TODO: maybe add back art :)
                    //                     eprintln!(
                    //                         "  \x1B[1m███████╗██╗ ██████╗
                    //   ██╔════╝██║██╔════╝
                    //   █████╗  ██║██║  ███╗
                    //   ██╔══╝  ██║██║   ██║
                    //   ██║     ██║╚██████╔╝
                    //   ╚═╝     ╚═╝ ╚═════╝ CLI\x1B[0m\n"
                    //                     );
                    println!(
                        "{}\n    {CLI_BINARY_NAME} [OPTIONS] [SUBCOMMAND]\n",
                        "USAGE:".bold().underlined(),
                    );
                    cmd.print_long_help()?;
                    Ok(())
                },
                CliRootCommands::Dashboard => launch_dashboard().await,
                CliRootCommands::Chat { input } => chat::chat(input.unwrap_or_default()).await,
            },
            // Root command
            None => launch_dashboard().await,
        }
    }

    async fn send_telemetry(&self) {
        match &self.subcommand {
            None
            | Some(
                CliRootCommands::Init(_)
                | CliRootCommands::Internal(_)
                | CliRootCommands::Completion(_)
                | CliRootCommands::Hook(_),
            ) => {},
            Some(subcommand) => {
                fig_telemetry::send_cli_subcommand_executed(subcommand.name()).await;
            },
        }
    }
}

async fn launch_dashboard() -> Result<()> {
    if manifest::is_minimal() || system_info::is_remote() {
        Cli::command().print_help()?;
        return Ok(());
    }

    launch_fig_desktop(LaunchArgs {
        wait_for_socket: true,
        open_dashboard: true,
        immediate_update: true,
        verbose: true,
    })?;

    let route = match is_logged_in().await {
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
            Cli::parse_from([CLI_BINARY_NAME, "restart", "app"]).subcommand,
            Some(CliRootCommands::Restart {
                process: Processes::App
            })
        );

        assert_eq!(
            Cli::parse_from([CLI_BINARY_NAME, "restart", "daemon"]).subcommand,
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
                CLI_BINARY_NAME,
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
    fn test_inline_shell_completion() {
        use internal::InternalSubcommand;

        assert_eq!(
            Cli::parse_from([CLI_BINARY_NAME, "_", "inline-shell-completion", "--buffer", ""]).subcommand,
            Some(CliRootCommands::Internal(InternalSubcommand::InlineShellCompletion {
                buffer: "".to_string()
            }))
        );

        assert_eq!(
            Cli::parse_from([CLI_BINARY_NAME, "_", "inline-shell-completion", "--buffer", "foo"]).subcommand,
            Some(CliRootCommands::Internal(InternalSubcommand::InlineShellCompletion {
                buffer: "foo".to_string()
            }))
        );

        assert_eq!(
            Cli::parse_from([CLI_BINARY_NAME, "_", "inline-shell-completion", "--buffer", "-"]).subcommand,
            Some(CliRootCommands::Internal(InternalSubcommand::InlineShellCompletion {
                buffer: "-".to_string()
            }))
        );

        assert_eq!(
            Cli::parse_from([CLI_BINARY_NAME, "_", "inline-shell-completion", "--buffer", "--"]).subcommand,
            Some(CliRootCommands::Internal(InternalSubcommand::InlineShellCompletion {
                buffer: "--".to_string()
            }))
        );

        assert_eq!(
            Cli::parse_from([CLI_BINARY_NAME, "_", "inline-shell-completion", "--buffer", "--foo bar"]).subcommand,
            Some(CliRootCommands::Internal(InternalSubcommand::InlineShellCompletion {
                buffer: "--foo bar".to_string()
            }))
        );

        assert_eq!(
            Cli::parse_from([
                CLI_BINARY_NAME,
                "_",
                "inline-shell-completion-accept",
                "--buffer",
                "abc",
                "--suggestion",
                "def"
            ])
            .subcommand,
            Some(CliRootCommands::Internal(
                InternalSubcommand::InlineShellCompletionAccept {
                    buffer: "abc".to_string(),
                    suggestion: "def".to_string()
                }
            ))
        );
    }
}
