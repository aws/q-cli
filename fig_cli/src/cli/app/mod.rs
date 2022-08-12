pub mod uninstall;

use std::iter::empty;
use std::process::Command;
use std::time::Duration;

use cfg_if::cfg_if;
use clap::Subcommand;
use crossterm::style::Stylize;
use eyre::Result;
use fig_ipc::command::{
    quit_command,
    update_command,
};
use fig_settings::{
    settings,
    state,
};
use regex::Regex;
use tracing::{
    info,
    trace,
};

use crate::cli::debug::get_app_info;
use crate::util::{
    is_app_running,
    launch_fig,
    LaunchOptions,
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
    /// Set the internal psudo-terminal path
    SetPath,
    /// Uninstall the Fig app
    Uninstall(uninstall::UninstallArgs),
    /// Prompts shown on terminal startup
    Prompts,
}

pub fn launch_fig_cli() -> Result<()> {
    if is_app_running() {
        println!("\n→ Fig is already running.\n");
        return Ok(());
    }

    launch_fig(LaunchOptions::new().wait_for_activation().verbose())?;
    Ok(())
}

pub async fn quit_fig() -> Result<()> {
    if !is_app_running() {
        println!("\n→ Fig is not running\n");
        return Ok(());
    }

    let telem_join = tokio::spawn(async {
        fig_telemetry::dispatch_emit_track(
            fig_telemetry::TrackEvent::new(
                fig_telemetry::TrackEventType::QuitApp,
                fig_telemetry::TrackSource::App,
                empty::<(&str, &str)>(),
            ),
            false,
        )
        .await
        .ok();
    });

    println!("\n→ Quitting Fig...\n");
    if quit_command().await.is_err() {
        tokio::time::sleep(Duration::from_millis(500)).await;
        let second_try = quit_command().await;
        if second_try.is_err() {
            if let Ok(info) = get_app_info() {
                let pid = Regex::new(r"pid = (\S+)")
                    .unwrap()
                    .captures(&info)
                    .and_then(|c| c.get(1));
                if let Some(pid) = pid {
                    let success = Command::new("kill")
                        .arg("-KILL")
                        .arg(pid.as_str())
                        .status()
                        .map(|res| res.success());
                    if let Ok(true) = success {
                        return Ok(());
                    }
                }
            }
            println!("\nUnable to quit Fig\n");
            second_try?;
        }
    }

    telem_join.await.ok();

    Ok(())
}

pub async fn restart_fig() -> Result<()> {
    if !is_app_running() {
        launch_fig_cli()
    } else {
        cfg_if! {
            if #[cfg(target_os = "linux")] {
                quit_fig().await?;
                launch_fig(LaunchOptions { wait_for_activation: true, verbose: true })?;
            } else {
                use fig_ipc::command::restart_command;

                println!("\n→ Restarting Fig...\n");
                if restart_command().await.is_err() {
                    println!("\nUnable to restart Fig\n");
                } else {
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                }
            }
        }
        Ok(())
    }
}

impl AppSubcommand {
    pub async fn execute(&self) -> Result<()> {
        match self {
            AppSubcommand::Install => {
                fig_ipc::command::run_install_script_command().await?;
            },
            AppSubcommand::Onboarding => {
                cfg_if! {
                    if #[cfg(unix)] {
                        use std::os::unix::process::CommandExt;

                        launch_fig(LaunchOptions::new().wait_for_activation().verbose())?;
                        if state::set_value("user.onboarding", true).is_ok() {
                            Command::new("bash")
                                .args(["-c", include_str!("onboarding.sh")])
                                .exec();
                        }
                    } else if #[cfg(windows)] {
                        println!("Onboarding isn't supported on Windows yet");
                    }
                }
            },
            AppSubcommand::Prompts => {
                if is_app_running() {
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
                            launch_fig(LaunchOptions::new().wait_for_activation()).ok();
                        } else {
                            trace!("autoupdates are disabled.");

                            println!("A new version of Fig is available. (Autoupdates are disabled)");
                            println!("To update, run: {}", "fig update".magenta());
                        }
                    }
                } else {
                    let no_autolaunch = settings::get_bool_or("app.disableAutolaunch", false);
                    let user_quit_app = state::get_bool_or("APP_TERMINATED_BY_USER", false);
                    if !no_autolaunch && !user_quit_app && !fig_util::in_ssh() {
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

                        launch_fig(LaunchOptions::new())?;
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
            AppSubcommand::Quit => quit_fig().await?,
            AppSubcommand::Launch => launch_fig_cli()?,
            AppSubcommand::Running => {
                println!("{}", if is_app_running() { "1" } else { "0" });
            },
            AppSubcommand::SetPath => {
                cfg_if! {
                    if #[cfg(unix)] {
                        use eyre::WrapErr;
                        use fig_ipc::hook::send_hook_to_socket;
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
