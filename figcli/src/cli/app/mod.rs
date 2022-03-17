pub mod uninstall;

use crate::{
    cli::debug::get_app_info,
    util::{launch_fig, LaunchOptions},
};

use anyhow::{Context, Result};
use clap::Subcommand;
use crossterm::style::Stylize;
use fig_ipc::{
    command::{quit_command, restart_command, update_command},
    hook::send_hook_to_socket,
};
use fig_proto::hooks;
use regex::Regex;
use serde_json::json;
use std::{process::Command, time::Duration};
use tracing::{info, trace};

use fig_settings::{settings, state};

#[derive(Debug, Subcommand)]
pub enum AppSubcommand {
    Install,
    Onboarding,
    Running,
    Launch,
    Restart,
    Quit,
    SetPath,
    Uninstall,
    Prompts,
}

fn is_app_running() -> bool {
    match get_app_info() {
        Ok(s) => !s.is_empty(),
        _ => false,
    }
}

pub fn launch_fig_cli() -> Result<()> {
    if is_app_running() {
        println!("\n→ Fig is already running.\n");
        return Ok(());
    }

    launch_fig(LaunchOptions {
        wait_for_activation: true,
        verbose: true,
    })?;
    Ok(())
}

pub async fn quit_fig() -> Result<()> {
    if !is_app_running() {
        println!("\n→ Fig is not running\n");
        return Ok(());
    }

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
            return second_try;
        }
    }
    Ok(())
}

pub async fn restart_fig() -> Result<()> {
    if !is_app_running() {
        launch_fig_cli()
    } else {
        println!("\n→ Restarting Fig...\n");
        if restart_command().await.is_err() {
            println!("\nUnable to restart Fig\n");
        } else {
            tokio::time::sleep(Duration::from_millis(1000)).await;
        }
        Ok(())
    }
}

impl AppSubcommand {
    pub async fn execute(&self) -> Result<()> {
        match self {
            AppSubcommand::Install => {
                fig_ipc::command::run_install_script_command().await?;
            }
            AppSubcommand::Onboarding => {
                launch_fig(LaunchOptions {
                    wait_for_activation: true,
                    verbose: true,
                })?;
                if state::set_value("user.onboarding", true).is_ok() {
                    Command::new("bash")
                        .args(["-c", include_str!("onboarding.sh")])
                        .spawn()?
                        .wait()?;
                }
            }
            AppSubcommand::Prompts => {
                if is_app_running() {
                    let new_version = state::get_string("NEW_VERSION_AVAILABLE")?;
                    if new_version.is_some() {
                        info!("New version {} is available", new_version.unwrap());
                        let no_autoupdates =
                            settings::get_bool("app.disableAutoupdates")?.unwrap_or(false);

                        if no_autoupdates {
                            trace!("autoupdates are disabled.");

                            println!(
                                "A new version of Fig is available. (Autoupdates are disabled)"
                            );
                        } else {
                            trace!("starting autoupdate");

                            println!("Updating {} to latest version...", "Fig".magenta());
                            let already_seen_hint: bool =
                                state::get_bool("DISPLAYED_AUTOUPDATE_SETTINGS_HINT")?
                                    .unwrap_or(false);

                            if !already_seen_hint {
                                println!("(To turn off automatic updates, run `fig settings app.disableAutoupdates true`)");
                                state::set_value("DISPLAYED_AUTOUPDATE_SETTINGS_HINT", true)?
                            }

                            // trigger forced update. This will QUIT the macOS app, it must be relaunched...
                            trace!("sending update commands to macOS app");
                            update_command(true).await?;

                            // Sleep for a bit
                            tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

                            trace!("launching updated version of Fig");
                            launch_fig(LaunchOptions {
                                wait_for_activation: true,
                                verbose: false,
                            })
                            .ok();
                        }
                    }
                } else {
                    let no_autolaunch =
                        settings::get_bool("app.disableAutolaunch")?.unwrap_or(false);
                    let user_quit_app = state::get_bool("APP_TERMINATED_BY_USER")?.unwrap_or(false);
                    if !no_autolaunch && !user_quit_app {
                        let already_seen_hint: bool =
                            fig_settings::state::get_bool("DISPLAYED_AUTOLAUNCH_SETTINGS_HINT")?
                                .unwrap_or(false);
                        println!("Launching {}...", "Fig".magenta());
                        if !already_seen_hint {
                            println!("(To turn off autolaunch, run `fig settings app.disableAutolaunch true`)");
                            fig_settings::state::set_value(
                                "DISPLAYED_AUTOLAUNCH_SETTINGS_HINT",
                                true,
                            )?
                        }

                        launch_fig(LaunchOptions {
                            wait_for_activation: false,
                            verbose: false,
                        })?;
                    }
                }
            }
            AppSubcommand::Uninstall => {
                uninstall::uninstall_mac_app().await;
            }
            AppSubcommand::Restart => restart_fig().await?,
            AppSubcommand::Quit => quit_fig().await?,
            AppSubcommand::Launch => launch_fig_cli()?,
            AppSubcommand::Running => {
                println!("{}", if is_app_running() { "1" } else { "0" });
            }
            AppSubcommand::SetPath => {
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
            }
        }
        Ok(())
    }
}
