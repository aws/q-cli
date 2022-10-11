use cfg_if::cfg_if;
use eyre::Result;

use crate::util::dialoguer_theme;

pub async fn uninstall_command(no_confirm: bool) -> Result<()> {
    if fig_util::system_info::in_wsl() {
        println!("Refer to your package manager in order to uninstall Fig from WSL");
        println!("If you're having issues uninstalling fig, run `fig issue`");
        return Ok(());
    }

    let should_uninstall = if no_confirm {
        true
    } else {
        dialoguer::Confirm::with_theme(&dialoguer_theme())
            .with_prompt("Are you sure you want to uninstall Fig?")
            .interact()?
    };

    if !should_uninstall {
        println!("Phew...");
        return Ok(());
    }

    cfg_if! {
        if #[cfg(unix)] {
            let emit = tokio::spawn(fig_telemetry::emit_track(fig_telemetry::TrackEvent::new(
                fig_telemetry::TrackEventType::UninstalledApp,
                fig_telemetry::TrackSource::Cli,
                env!("CARGO_PKG_VERSION").into(),
                std::iter::empty::<(&str, &str)>(),
            )));
            let (emit_join, uninstall_join) = tokio::join!(emit, uninstall());
            emit_join?.ok();
            uninstall_join?;
        } else if #[cfg(target_os = "windows")] {
            println!("Please uninstall fig from the `Add or remove programs` menu for now.");
            println!("If you're having issues uninstalling fig, run `fig issue` to let us know, and use the tool at the following link to remove fig:");
            println!("https://support.microsoft.com/en-us/topic/fix-problems-that-block-programs-from-being-installed-or-removed-cca7d1b6-65a9-3d98-426b-e9f927e1eb4d")
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
async fn uninstall() -> Result<()> {
    use fig_install::InstallComponents;
    use tokio::process::Command;
    use tracing::warn;

    let url = fig_install::get_uninstall_url();
    fig_util::open_url(url).ok();
    fig_install::uninstall(InstallComponents::all()).await?;

    // TODO(sean)
    // 1. Remove login items
    // 2. Set title of running ttys "Restart this terminal to finish uninstalling Fig..."
    // 3. Delete webview cache

    if let Err(err) = Command::new("killall").args(["fig_desktop"]).output().await {
        warn!("Failed to quit running Fig app: {err}");
    }
    Ok(())
}

#[cfg(target_os = "linux")]
async fn uninstall() -> Result<()> {
    use std::env;
    use std::process::Command;

    use fig_util::manifest::{
        self,
        ManagedBy,
    };

    if nix::unistd::getuid().is_root() {
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
    } else if which::which("sudo").is_ok() {
        // note: this does not trigger a race condition because any user that can replace fig_cli could just
        // replace it with a malicious executable before we are even run
        Command::new("sudo")
            .arg(std::env::current_exe()?)
            .arg("uninstall")
            .arg("-y")
            .status()?;
    } else {
        eyre::bail!("This command must be run as root");
    }

    println!("Goodbye!");

    Ok(())
}

#[cfg(target_os = "linux")]
mod linux {
    use eyre::Result;

    pub async fn uninstall_apt(pkg: String) -> Result<()> {
        tokio::process::Command::new("apt")
            .arg("remove")
            .arg("-y")
            .arg(pkg)
            .status()
            .await?;
        std::fs::remove_file("/etc/apt/sources.list.d/fig.list")?;
        std::fs::remove_file("/etc/apt/keyrings/fig.gpg")?;

        Ok(())
    }

    pub async fn uninstall_dnf(pkg: String) -> Result<()> {
        tokio::process::Command::new("dnf")
            .arg("remove")
            .arg("-y")
            .arg(pkg)
            .status()
            .await?;
        std::fs::remove_file("/etc/yum.repos.d/fig.repo")?;

        Ok(())
    }

    pub async fn uninstall_pacman(pkg: String) -> Result<()> {
        tokio::process::Command::new("pacman")
            .arg("-Rs")
            .arg(pkg)
            .status()
            .await?;

        Ok(())
    }
}
