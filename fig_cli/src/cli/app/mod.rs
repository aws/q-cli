use std::io::Write;
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
    is_codewhisperer_desktop_running,
    manifest,
};
use tracing::{
    error,
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
    /// Remove Fig daemon
    #[arg(long)]
    pub daemon: bool,
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
    /// Install the Fig app
    Install,
    /// Run the Fig tutorial again
    Onboarding,
    /// Check if Fig is running
    Running,
    /// Launch the Fig desktop app
    Launch,
    /// Restart the Fig desktop app
    Restart,
    /// Quit the Fig desktop app
    Quit,
    /// Set the internal pseudo-terminal path
    #[deprecated]
    SetPath,
    /// Uninstall the Fig app
    Uninstall(UninstallArgs),
    /// Prompts shown on terminal startup
    Prompts,
}

impl From<&UninstallArgs> for InstallComponents {
    fn from(args: &UninstallArgs) -> Self {
        if args.input_method || args.dotfiles || args.ssh || args.daemon || args.app_bundle {
            let mut flags = InstallComponents::empty();
            flags.set(InstallComponents::INPUT_METHOD, args.input_method);
            flags.set(InstallComponents::SHELL_INTEGRATIONS, args.dotfiles);
            flags.set(InstallComponents::SSH, args.ssh);
            flags.set(InstallComponents::DAEMON, args.daemon);
            flags.set(InstallComponents::DESKTOP_APP, args.app_bundle);
            flags
        } else {
            InstallComponents::all()
        }
    }
}

pub async fn restart_fig() -> Result<()> {
    if fig_util::system_info::is_remote() {
        bail!("Please restart Fig from your host machine");
    }

    if !is_codewhisperer_desktop_running() {
        launch_fig_desktop(LaunchArgs {
            wait_for_socket: true,
            open_dashboard: false,
            immediate_update: true,
            verbose: true,
        })?;

        Ok(())
    } else {
        println!("Restarting CodeWhisperer");
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
   * Fig not working? Run {}
                                ",
                                "Fig Autocomplete".bold(),
                                "cw".bold().magenta(),
                                "cw doctor".bold().magenta(),
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
                                "cw doctor".bold().magenta()
                            );
                        }
                    }
                }
            },
            AppSubcommand::Prompts => {
                if fig_util::manifest::is_headless() {
                    if let Ok(Some(version)) = state::get_string("update.latestVersion") {
                        writeln!(
                            std::io::stdout(),
                            "A new version ({version}) of Fig is available! Please update from your package manager."
                        )
                        .ok();
                    }

                    match fig_install::check_for_updates(false).await {
                        Ok(Some(package)) => {
                            let _ = state::set_value("update.latestVersion", package.version);
                        },
                        Ok(None) => {
                            // no version available
                            let _ = state::remove_value("update.latestVersion");
                        },
                        Err(err) => error!(%err, "Failed checking for updates"),
                    }
                } else if is_codewhisperer_desktop_running() {
                    let new_version = state::get_string("NEW_VERSION_AVAILABLE").ok().flatten();
                    if let Some(version) = new_version {
                        info!("New version {} is available", version);
                        let autoupdates = !settings::get_bool_or("app.disableAutoupdates", false);

                        if autoupdates {
                            trace!("starting autoupdate");

                            println!("Updating {} to latest version...", "CodeWhisperer".magenta());
                            let already_seen_hint = state::get_bool_or("DISPLAYED_AUTOUPDATE_SETTINGS_HINT", false);

                            if !already_seen_hint {
                                println!(
                                    "(To turn off automatic updates, run {})",
                                    "fig settings app.disableAutoupdates true".magenta()
                                );
                                state::set_value("DISPLAYED_AUTOUPDATE_SETTINGS_HINT", true)?
                            }

                            // trigger forced update. This will QUIT the macOS app, it must be relaunched...
                            trace!("sending update commands to macOS app");
                            update_command(true).await?;

                            // Sleep for a bit
                            tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

                            trace!("launching updated version of Fig");
                            launch_fig_desktop(LaunchArgs {
                                wait_for_socket: true,
                                open_dashboard: false,
                                immediate_update: true,
                                verbose: false,
                            })
                            .ok();
                        } else {
                            trace!("autoupdates are disabled.");

                            println!("A new version of Fig is available. (Autoupdates are disabled)");
                            println!("To update, run: {}", "fig update".magenta());
                        }
                    }
                } else {
                    let no_autolaunch =
                        settings::get_bool_or("app.disableAutolaunch", false) || manifest::is_headless();
                    let user_quit_app = state::get_bool_or("APP_TERMINATED_BY_USER", false);
                    if !no_autolaunch && !user_quit_app && !fig_util::system_info::in_ssh() {
                        let already_seen_hint: bool =
                            fig_settings::state::get_bool_or("DISPLAYED_AUTOLAUNCH_SETTINGS_HINT", false);
                        println!("Launching {}...", "Fig".magenta());
                        if !already_seen_hint {
                            println!(
                                "(To turn off autolaunch, run {})",
                                "fig settings app.disableAutolaunch true".magenta()
                            );
                            fig_settings::state::set_value("DISPLAYED_AUTOLAUNCH_SETTINGS_HINT", true)?
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
                            let url = fig_install::get_uninstall_url(false);
                            fig_util::open_url_async(url).await.ok();
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
                if is_codewhisperer_desktop_running() {
                    println!("Fig is already running!");
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
                println!("{}", if is_codewhisperer_desktop_running() { "1" } else { "0" });
            },
            #[allow(deprecated)]
            AppSubcommand::SetPath => {},
        }
        Ok(())
    }
}
