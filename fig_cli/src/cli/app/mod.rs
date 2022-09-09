pub mod uninstall;

use std::io::Write;
use std::str::FromStr;

use cfg_if::cfg_if;
use clap::Subcommand;
use crossterm::style::Stylize;
use eyre::{
    bail,
    Result,
};
use fig_ipc::local::update_command;
use fig_settings::{
    settings,
    state,
};
use fig_update::index::local_manifest_version;
use fig_util::manifest;
use tracing::{
    error,
    info,
    trace,
};

use crate::util::{
    is_app_running,
    launch_fig,
    LaunchArgs,
};

#[derive(Debug, Subcommand)]
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
    SetPath,
    /// Uninstall the Fig app
    Uninstall(uninstall::UninstallArgs),
    /// Prompts shown on terminal startup
    Prompts,
}

pub async fn restart_fig() -> Result<()> {
    if fig_util::system_info::is_remote() {
        bail!("Please restart Fig from your host machine");
    }

    if !is_app_running() {
        launch_fig(LaunchArgs {
            print_running: false,
            print_launching: true,
            wait_for_launch: true,
        })
    } else {
        cfg_if! {
            if #[cfg(target_os = "linux")] {
                crate::util::quit_fig().await?;
                launch_fig(LaunchArgs {
                    print_running: false,
                    print_launching: true,
                    wait_for_launch: true
                })?;
            } else {
                use eyre::Context;

                use fig_ipc::local::restart_command;

                println!("Restarting Fig");
                restart_command().await.context("Unable to restart Fig")?;
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            }
        }

        Ok(())
    }
}

impl AppSubcommand {
    pub async fn execute(&self) -> Result<()> {
        match self {
            AppSubcommand::Install => {
                fig_ipc::local::run_install_script_command().await?;
            },
            AppSubcommand::Onboarding => {
                cfg_if! {
                    if #[cfg(unix)] {
                        use std::process::Command;
                        use std::os::unix::process::CommandExt;

                        launch_fig(LaunchArgs {
                            print_running: false,
                            print_launching: true,
                            wait_for_launch: true
                        })?;
                        if state::set_value("user.onboarding", true).is_ok() {
                            Command::new("bash")
                                .args(["-c", include_str!("onboarding.sh")])
                                .exec();
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
                                "fig doctor".bold().magenta()
                            );
                        }
                    }
                }
            },
            AppSubcommand::Prompts => {
                if fig_util::manifest::is_headless() {
                    if let Ok(Some(version)) = state::get_string("update.latestVersion") {
                        if let Ok(local_version) = local_manifest_version() {
                            if let Ok(remote_version) = semver::Version::from_str(&version) {
                                if local_version < remote_version {
                                    writeln!(
                                        std::io::stdout(),
                                        "A new version of Fig is available! Please update from your package manager."
                                    )
                                    .ok();
                                }
                            }
                        }
                    }

                    match fig_update::check_for_updates(None).await {
                        Ok(Some(remote)) => {
                            state::set_value("update.latestVersion", remote.version).ok();
                        },
                        Ok(None) => {}, // no version available
                        Err(err) => error!(%err, "Failed checking for updates"),
                    }
                } else if is_app_running() {
                    let new_version = state::get_string("NEW_VERSION_AVAILABLE").ok().flatten();
                    if let Some(version) = new_version {
                        info!("New version {} is available", version);
                        let autoupdates = !settings::get_bool_or("app.disableAutoupdates", false);

                        if autoupdates {
                            trace!("starting autoupdate");

                            println!("Updating {} to latest version...", "Fig".magenta());
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
                            launch_fig(LaunchArgs {
                                print_running: false,
                                print_launching: false,
                                wait_for_launch: true,
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

                        launch_fig(LaunchArgs {
                            print_running: false,
                            print_launching: false,
                            wait_for_launch: false,
                        })?;
                    }
                }
            },
            AppSubcommand::Uninstall(args) => {
                cfg_if! {
                    if #[cfg(target_os = "macos")] {
                        uninstall::uninstall_mac_app(args).await;
                    } else {
                        let _args = args;
                        eyre::bail!("Unable to uninstall app via `fig app uninstall` on {}", std::env::consts::OS)
                    }
                }
            },
            AppSubcommand::Restart => restart_fig().await?,
            AppSubcommand::Quit => crate::util::quit_fig().await?,
            AppSubcommand::Launch => launch_fig(LaunchArgs {
                print_running: true,
                print_launching: true,
                wait_for_launch: true,
            })?,
            AppSubcommand::Running => {
                println!("{}", if is_app_running() { "1" } else { "0" });
            },
            AppSubcommand::SetPath => {
                cfg_if! {
                    if #[cfg(unix)] {
                        use std::process::Command;

                        use eyre::WrapErr;
                        use fig_ipc::local::send_hook_to_socket;
                        use fig_proto::hooks;
                        use serde_json::json;

                        println!("\nSetting $PATH variable in Fig pseudo-terminal...\n");
                        let path = std::env::var("PATH")?;
                        fig_settings::state::set_value("pty.path", json!(path))?;
                        println!(
                            "Fig will now use the following path to locate the fig executable:\n{}\n",
                            path.magenta()
                        );


                        let output = Command::new("tty").output().context(format!(
                            "{} Unable to reload. Restart terminal to apply changes.",
                            "Error:".red()
                        ))?;


                        let tty = String::from_utf8(output.stdout)?;
                        let pid = nix::unistd::getppid();

                        let hook = hooks::generate_shell_context(pid, tty, None, None)
                            .and_then(hooks::new_init_hook)
                            .context(format!(
                                "{} Unable to reload. Restart terminal to apply changes.",
                                "Error:".red()
                            ))?;

                        send_hook_to_socket(hook).await.context(format!(
                            "\n{}\nFig might not be running to launch Fig run: {}\n",
                            "Unable to Connect to Fig:".bold(),
                            "fig launch".magenta()
                        ))?;
                    } else {
                        eyre::bail!("Not implemented on this platform");
                    }
                }
            },
        }
        Ok(())
    }
}
