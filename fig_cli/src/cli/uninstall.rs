use cfg_if::cfg_if;
use eyre::Result;

use crate::util::dialoguer_theme;

pub async fn uninstall_command() -> Result<()> {
    let should_uninstall = dialoguer::Confirm::with_theme(&dialoguer_theme())
        .with_prompt("Are you sure you want to uninstall Fig?")
        .interact()?;

    if !should_uninstall {
        println!("Phew...");
        return Ok(());
    }

    cfg_if! {
        if #[cfg(target_os = "linux")] {
            uninstall().await?;
        } else if #[cfg(target_os = "macos")] {
            if super::desktop_app_is_installed() {
                use crate::util::{
                    launch_fig,
                    LaunchOptions,
                };
                let success = if launch_fig(LaunchOptions::new().wait_for_activation().verbose()).is_ok() {
                    fig_ipc::command::uninstall_command().await.is_ok()
                } else {
                    false
                };

                if !success {
                    println!("Fig is not running. Please launch Fig and try again to complete uninstall.");
                }
            } else {
                super::installation::uninstall_cli(super::installation::InstallComponents::all())?
            }
        } else if #[cfg(target_os = "windows")] {
            println!("Please uninstall fig from the `Add or remove programs` menu for now.\n");
            println!("If you're having issues uninstalling fig, run `fig issue` to let us know, and use the tool at the following link to remove fig:");
            println!("https://support.microsoft.com/en-us/topic/fix-problems-that-block-programs-from-being-installed-or-removed-cca7d1b6-65a9-3d98-426b-e9f927e1eb4d")
        }
    }

    Ok(())
}

#[cfg(target_os = "linux")]
async fn uninstall() -> Result<()> {
    use std::env;
    use std::process::Command;

    if !nix::unistd::getuid().is_root() {
        eyre::bail!("This command must be run as root");
    }

    let package_name = env::var("FIG_PACKAGE_NAME").unwrap_or_else(|_| {
        if super::desktop_app_is_installed() {
            "fig"
        } else {
            "fig-headless"
        }
        .to_owned()
    });

    let package_manager = env::var("FIG_PACKAGE_MANAGER").or_else(|_| {
        Ok(if which::which("apt").is_ok() {
            "apt"
        } else if which::which("dnf").is_ok() {
            "dnf"
        } else if which::which("pacman").is_ok() {
            "pacman"
        } else {
            eyre::bail!("Couldn't detect a supported package manager.");
        }
        .to_string())
    })?;

    Command::new("killall").arg("fig_desktop").status()?;

    match package_manager.as_str() {
        "apt" => linux::uninstall_apt(package_name).await?,
        "dnf" => linux::uninstall_dnf(package_name).await?,
        "pacman" => linux::uninstall_pacman(package_name).await?,
        _ => {
            eyre::bail!("Unknown package manager.");
        },
    }

    println!("Goodbye!");

    Ok(())
}

#[cfg(target_os = "linux")]
mod linux {
    use std::process::Command;

    use eyre::Result;

    pub async fn uninstall_apt(pkg: String) -> Result<()> {
        Command::new("apt").arg("remove").arg("-y").arg(pkg).status()?;
        std::fs::remove_file("/etc/apt/sources.list.d/fig.list")?;
        std::fs::remove_file("/etc/apt/keyrings/fig.gpg")?;

        Ok(())
    }

    pub async fn uninstall_dnf(pkg: String) -> Result<()> {
        Command::new("dnf").arg("remove").arg("-y").arg(pkg).status()?;
        std::fs::remove_file("/etc/yum.repos.d/fig.repo")?;

        Ok(())
    }

    pub async fn uninstall_pacman(pkg: String) -> Result<()> {
        Command::new("pacman").arg("-Rs").arg(pkg).status()?;

        Ok(())
    }
}
