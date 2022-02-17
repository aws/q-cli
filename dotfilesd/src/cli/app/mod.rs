use crate::{cli::debug::get_app_info, util::settings::Settings};

use anyhow::{Context, Result};
use clap::Subcommand;
use crossterm::style::Stylize;
use fig_ipc::{
    command::{quit_command, restart_command},
    hook::{create_init_hook, send_hook_to_socket},
};
use regex::Regex;
use serde_json::json;
use std::{process::Command, time::Duration};

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

fn launch_fig() -> Result<()> {
    if is_app_running() {
        println!("\n→ Fig is already running.\n");
        return Ok(());
    }

    println!("\n→ Launching Fig...\n");
    Command::new("open")
        .args(["-g", "-b", "com.mschrage.fig"])
        .spawn()
        .context("\n→ Fig could not be launched.\n")?;
    Ok(())
}

impl AppSubcommand {
    pub async fn execute(&self) -> Result<()> {
        match self {
            AppSubcommand::Install => {
                Command::new("bash")
                    .args(["-c", include_str!("install_and_upgrade.sh")])
                    .spawn()?
                    .wait()?;
            }
            AppSubcommand::Onboarding => {
                Command::new("bash")
                    .args(["-c", include_str!("onboarding.sh")])
                    .spawn()?
                    .wait()?;
            }
            AppSubcommand::Prompts => {
                Command::new("bash")
                    .args(["-c", include_str!("prompts.sh")])
                    .spawn()?
                    .wait()?;
            }
            AppSubcommand::Uninstall => {
                Command::new("bash")
                    .args(["-c", include_str!("uninstall-script.sh")])
                    .spawn()?
                    .wait()?;
            }
            AppSubcommand::Restart => {
                if restart_command().await.is_err() {
                    launch_fig()?
                } else {
                    println!("\n→ Restarting Fig...\n");
                }
            }
            AppSubcommand::Quit => {
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
                                .map(|c| c.get(1))
                                .flatten();
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
            }
            AppSubcommand::Launch => launch_fig()?,
            AppSubcommand::Running => {
                println!("{}", if is_app_running() { "1" } else { "0" });
            }
            AppSubcommand::SetPath => {
                println!("\nSetting $PATH variable in Fig pseudo-terminal...\n");
                let path = std::env::var("PATH")?;
                let result = Settings::set("pty.path", json!(path));

                if result.is_err() {
                    println!("{} Unable to load settings file", "Error:".red());
                    return result;
                }
                println!(
                    "Fig will now use the following path to locate the fig executable:\n{}\n",
                    path.magenta()
                );
                let output = Command::new("tty").output().context(format!(
                    "{} Unable to reload. Restart terminal to apply changes.",
                    "Error:".red()
                ))?;
                let tty = String::from_utf8(output.stdout)?;
                let hook =
                    create_init_hook(nix::unistd::getppid().into(), tty).context(format!(
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
