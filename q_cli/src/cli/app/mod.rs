use std::time::Duration;

use cfg_if::cfg_if;
use clap::{
    arg,
    Args,
    Subcommand,
};
use crossterm::style::Stylize;
use eyre::{
    bail,
    Result,
};
use fig_install::InstallComponents;
use fig_ipc::local::update_command;
use fig_settings::{
    settings,
    state,
};
use fig_util::desktop::{
    launch_fig_desktop,
    LaunchArgs,
};
use fig_util::{
    desktop_app_running,
    manifest,
    CLI_BINARY_NAME,
    PRODUCT_NAME,
};
use tracing::{
    info,
    trace,
};

#[derive(Debug, Args, PartialEq, Eq)]
pub struct UninstallArgs {
    /// Remove executable and user data
    #[arg(long)]
    pub app_bundle: bool,
    /// Remove input method
    #[arg(long)]
    pub input_method: bool,
    /// Remove dotfile shell integration
    #[arg(long)]
    pub dotfiles: bool,
    /// Remove SSH integration
    #[arg(long)]
    pub ssh: bool,
    /// Do not open the uninstallation page
    #[arg(long)]
    pub no_open: bool,
    /// Only open the uninstallation page
    #[arg(long)]
    pub only_open: bool,
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum AppSubcommand {
    /// Install the app
    Install,
    /// Run the tutorial again
    Onboarding,
    /// Check if the desktop app is running
    Running,
    /// Launch the desktop app
    Launch,
    /// Restart the desktop app
    Restart,
    /// Quit the desktop app
    Quit,
    /// Set the internal pseudo-terminal path
    #[deprecated]
    SetPath,
    /// Uninstall the desktop app
    Uninstall(UninstallArgs),
    /// Prompts shown on terminal startup
    Prompts,
}

impl From<&UninstallArgs> for InstallComponents {
    fn from(args: &UninstallArgs) -> Self {
        if args.input_method || args.dotfiles || args.ssh || args.app_bundle {
            let mut flags = InstallComponents::empty();
            flags.set(InstallComponents::INPUT_METHOD, args.input_method);
            flags.set(InstallComponents::SHELL_INTEGRATIONS, args.dotfiles);
            flags.set(InstallComponents::SSH, args.ssh);
            flags.set(InstallComponents::DESKTOP_APP, args.app_bundle);
            flags
        } else {
            InstallComponents::all()
        }
    }
}

pub async fn restart_fig() -> Result<()> {
    if fig_util::system_info::is_remote() {
        bail!("Please restart {PRODUCT_NAME} from your host machine");
    }

    if !desktop_app_running() {
        launch_fig_desktop(LaunchArgs {
            wait_for_socket: true,
            open_dashboard: false,
            immediate_update: true,
            verbose: true,
        })?;

        Ok(())
    } else {
        println!("Restarting {PRODUCT_NAME}");
        crate::util::quit_fig(false).await?;
        tokio::time::sleep(Duration::from_millis(1000)).await;
        launch_fig_desktop(LaunchArgs {
            wait_for_socket: true,
            open_dashboard: false,
            immediate_update: true,
            verbose: false,
        })?;

        Ok(())
    }
}

impl AppSubcommand {
    pub async fn execute(&self) -> Result<()> {
        match self {
            AppSubcommand::Install => {
                // TODO(sean) install MacOS specific script
            },
            AppSubcommand::Onboarding => {
                cfg_if! {
                    if #[cfg(unix)] {
                        launch_fig_desktop(LaunchArgs {
                            wait_for_socket: true,
                            open_dashboard: false,
                            immediate_update: true,
                            verbose: true,
                        })?;

                        if state::set_value("user.onboarding", true).is_ok() && state::set_value("doctor.prompt-restart-terminal", false).is_ok() {
                            // Command::new("bash")
                            //     .args(["-c", include_str!("onboarding.sh")])
                            //     .exec();
                            println!(
                                "
   ███████╗██╗ ██████╗
   ██╔════╝██║██╔════╝
   █████╗  ██║██║  ███╗
   ██╔══╝  ██║██║   ██║
   ██║     ██║╚██████╔╝
   ╚═╝     ╚═╝ ╚═════╝  ....is now installed!

   Start typing to use {}

   * Change settings? Run {}
   * {PRODUCT_NAME} not working? Run {}
                                ",
                                format!("{PRODUCT_NAME} Autocomplete").bold(),
                                CLI_BINARY_NAME.bold().magenta(),
                                format!("{CLI_BINARY_NAME} doctor").bold().magenta(),
                            );
                        }
                    } else if #[cfg(windows)] {
                        if state::set_value("user.onboarding", true).is_ok() &&
                           state::set_value("doctor.prompt-restart-terminal", false).is_ok() {
                            println!(
                                "

  \x1B[1m███████╗██╗ ██████╗
  ██╔════╝██║██╔════╝
  █████╗  ██║██║  ███╗
  ██╔══╝  ██║██║   ██║
  ██║     ██║╚██████╔╝
  ╚═╝     ╚═╝ ╚═════╝ Autocomplete\x1B[0m

1. Type {} and suggestions will appear.

2. Run {} to check for common bugs.

",
                                "\"cd \"".bold(),
                                format!("{CLI_BINARY_NAME} doctor").bold().magenta()
                            );
                        }
                    }
                }
            },
            AppSubcommand::Prompts => {
                if fig_util::manifest::is_minimal() {
                } else if desktop_app_running() {
                    let new_version = state::get_string("NEW_VERSION_AVAILABLE").ok().flatten();
                    if let Some(version) = new_version {
                        info!("New version {} is available", version);
                        let autoupdates = !settings::get_bool_or("app.disableAutoupdates", false);

                        if autoupdates {
                            trace!("starting autoupdate");

                            println!("Updating {} to latest version...", PRODUCT_NAME.magenta());
                            let already_seen_hint = state::get_bool_or("DISPLAYED_AUTOUPDATE_SETTINGS_HINT", false);

                            if !already_seen_hint {
                                println!(
                                    "(To turn off automatic updates, run {})",
                                    "fig settings app.disableAutoupdates true".magenta()
                                );
                                state::set_value("DISPLAYED_AUTOUPDATE_SETTINGS_HINT", true)?;
                            }

                            // trigger forced update. This will QUIT the macOS app, it must be relaunched...
                            trace!("sending update commands to macOS app");
                            update_command(true).await?;

                            // Sleep for a bit
                            tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

                            trace!("launching updated version");
                            launch_fig_desktop(LaunchArgs {
                                wait_for_socket: true,
                                open_dashboard: false,
                                immediate_update: true,
                                verbose: false,
                            })
                            .ok();
                        } else {
                            trace!("autoupdates are disabled.");

                            println!("A new version of {PRODUCT_NAME} is available. (Autoupdates are disabled)");
                            println!("To update, run: {}", "fig update".magenta());
                        }
                    }
                } else {
                    let no_autolaunch = settings::get_bool_or("app.disableAutolaunch", false) || manifest::is_minimal();
                    let user_quit_app = state::get_bool_or("APP_TERMINATED_BY_USER", false);
                    if !no_autolaunch && !user_quit_app && !fig_util::system_info::in_ssh() {
                        let already_seen_hint: bool =
                            fig_settings::state::get_bool_or("DISPLAYED_AUTOLAUNCH_SETTINGS_HINT", false);
                        println!("Launching {}...", PRODUCT_NAME.magenta());
                        if !already_seen_hint {
                            println!(
                                "(To turn off autolaunch, run {})",
                                "fig settings app.disableAutolaunch true".magenta()
                            );
                            fig_settings::state::set_value("DISPLAYED_AUTOLAUNCH_SETTINGS_HINT", true)?;
                        }

                        launch_fig_desktop(LaunchArgs {
                            wait_for_socket: false,
                            open_dashboard: false,
                            immediate_update: true,
                            verbose: false,
                        })?;
                    }
                }
            },
            AppSubcommand::Uninstall(args) => {
                cfg_if! {
                    if #[cfg(target_os = "macos")] {
                        // use fig_telemetry::{TrackSource, TrackEvent, TrackEventType};

                        // let telem_join = tokio::spawn(fig_telemetry::emit_track(TrackEvent::new(
                        //     TrackEventType::UninstalledApp,
                        //     TrackSource::Cli,
                        //     env!("CARGO_PKG_VERSION").into(),
                        //     [("source", "fig app uninstall")],
                        // )));

                        if !args.no_open && !crate::util::is_brew_reinstall().await {
                            fig_util::open_url_async(fig_install::UNINSTALL_URL).await.ok();
                        }

                        // telem_join.await.ok();

                        if !args.only_open {
                            fig_install::uninstall(args.into()).await?;
                        }
                    } else {
                        let _args = args;
                        eyre::bail!("Unable to uninstall app via `fig app uninstall` on {}", std::env::consts::OS)
                    }
                }
            },
            AppSubcommand::Restart => restart_fig().await?,
            AppSubcommand::Quit => crate::util::quit_fig(true).await?,
            AppSubcommand::Launch => {
                if desktop_app_running() {
                    println!("CodeWhisperer is already running!");
                    return Ok(());
                }

                launch_fig_desktop(LaunchArgs {
                    wait_for_socket: true,
                    open_dashboard: false,
                    immediate_update: true,
                    verbose: true,
                })?;
            },
            AppSubcommand::Running => {
                println!("{}", if desktop_app_running() { "1" } else { "0" });
            },
            #[allow(deprecated)]
            AppSubcommand::SetPath => {},
        }
        Ok(())
    }
}
