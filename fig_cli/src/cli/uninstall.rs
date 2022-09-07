use cfg_if::cfg_if;
use eyre::Result;

use crate::util::dialoguer_theme;

pub async fn uninstall_command() -> Result<()> {
    if fig_util::system_info::in_wsl() {
        println!("Refer to your package manager in order to uninstall Fig from WSL");
        println!("If you're having issues uninstalling fig, run `fig issue`");
        return Ok(());
    }

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
            if !fig_util::metadata::is_headless() {
                use crate::util::{LaunchArgs, launch_fig};
                let success = if launch_fig(LaunchArgs {
                    print_running: false,
                    print_launching: true,
                    wait_for_launch: true,
                })
                .is_ok()
                {
                    fig_ipc::local::uninstall_command().await.is_ok()
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
            println!("Please uninstall fig from the `Add or remove programs` menu for now.");
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

    use crate::util::manifest::{
        self,
        ManagedBy,
    };

    if !nix::unistd::getuid().is_root() {
        eyre::bail!("This command must be run as root");
    }

    let package_name = env::var("FIG_PACKAGE_NAME").unwrap_or_else(|_| {
        if !manifest::is_headless() {
            "fig"
        } else {
            "fig-headless"
        }
        .to_owned()
    });

    let package_manager = &manifest::manifest()
        .as_ref()
        .ok_or_else(|| eyre::eyre!("Failed getting installation manifest"))?
        .managed_by;

    Command::new("killall").arg("fig_desktop").status()?;

    match package_manager {
        ManagedBy::Apt => linux::uninstall_apt(package_name).await?,
        ManagedBy::Dnf => linux::uninstall_dnf(package_name).await?,
        ManagedBy::Pacman => linux::uninstall_pacman(package_name).await?,
        ManagedBy::Other(mgr) => {
            eyre::bail!("Unknown package manager {mgr}");
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
