pub mod uninstall;

use crate::{cli::debug::get_app_info, util::launch_fig};

use anyhow::{Context, Result};
use clap::Subcommand;
use crossterm::style::Stylize;
use fig_ipc::{
    command::{quit_command, restart_command},
    hook::send_hook_to_socket,
};
use fig_proto::hooks;
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

pub fn launch_fig_cli() -> Result<()> {
    if is_app_running() {
        println!("\n→ Fig is already running.\n");
        return Ok(());
    }

    launch_fig()?;
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
                launch_fig()?;
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
