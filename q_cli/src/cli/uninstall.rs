use cfg_if::cfg_if;
use crossterm::style::Stylize;
use eyre::Result;
use fig_util::{
    CLI_BINARY_NAME,
    PRODUCT_NAME,
};

use crate::util::dialoguer_theme;

pub async fn uninstall_command(no_confirm: bool) -> Result<()> {
    if fig_util::system_info::in_wsl() {
        println!("Refer to your package manager in order to uninstall {PRODUCT_NAME} from WSL");
        println!("If you're having issues uninstalling fig, run `{CLI_BINARY_NAME} issue`");
        return Ok(());
    }

    if !no_confirm {
        println!(
            "\nIs {PRODUCT_NAME} not working? Try running {}\n",
            format!("{CLI_BINARY_NAME} doctor").bold().magenta()
        );
        let should_continue = dialoguer::Select::with_theme(&dialoguer_theme())
            .with_prompt(format!("Are you sure want to continue uninstalling {PRODUCT_NAME}?"))
            .items(&["Yes", "No"])
            .default(0)
            .interact_opt()?;

        if should_continue == Some(0) {
            println!("Uninstalling {PRODUCT_NAME}");
        } else {
            println!("Cancelled");
            return Ok(());
        }
    };

    cfg_if! {
        if #[cfg(unix)] {
            // fig_telemetry::emit_track(fig_telemetry::TrackEvent::new(
            //     fig_telemetry::TrackEventType::UninstalledApp,
            //     fig_telemetry::TrackSource::Cli,
            //     env!("CARGO_PKG_VERSION").into(),
            //     std::iter::empty::<(&str, &str)>(),
            // )).await.ok();
            uninstall().await?;
        } else if #[cfg(target_os = "windows")] {
            println!("Please uninstall {PRODUCT_NAME} from the `Add or remove programs` menu for now.");
            println!("If you're having issues uninstalling {PRODUCT_NAME}, run `{CLI_BINARY_NAME} issue` to let us know, and use the tool at the following link to remove {PRODUCT_NAME}:");
            println!("https://support.microsoft.com/en-us/topic/fix-problems-that-block-programs-from-being-installed-or-removed-cca7d1b6-65a9-3d98-426b-e9f927e1eb4d")
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
async fn uninstall() -> Result<()> {
    fig_util::open_url(fig_install::UNINSTALL_URL).ok();
    auth::logout().await.ok();
    fig_install::uninstall(fig_install::InstallComponents::all()).await?;

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
    use fig_util::CLI_BINARY_NAME_MINIMAL;

    if nix::unistd::getuid().is_root() {
        let package_name = env::var("Q_PACKAGE_NAME").unwrap_or_else(|_| {
            match manifest::is_minimal() {
                true => CLI_BINARY_NAME_MINIMAL,
                false => CLI_BINARY_NAME,
            }
            .to_owned()
        });

        let package_manager = &manifest::manifest().managed_by;

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
        // note: this does not trigger a race condition because any user that can replace q_cli could just
        // replace it with a malicious executable before we are even run
        Command::new("sudo")
            .arg(std::env::current_exe()?)
            .arg("uninstall")
            .arg("-y")
            .status()?;
    } else {
        eyre::bail!("This command must be run as root");
    }

    Ok(())
}

#[cfg(all(unix, not(any(target_os = "macos", target_os = "linux"))))]
async fn uninstall() -> Result<()> {
    eyre::bail!("Guided uninstallation is not supported on this platform. Please uninstall manually.");
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
