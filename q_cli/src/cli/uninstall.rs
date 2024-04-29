use std::process::ExitCode;

use crossterm::style::Stylize;
use eyre::Result;
use fig_util::{
    CLI_BINARY_NAME,
    PRODUCT_NAME,
};

use crate::util::dialoguer_theme;

pub async fn uninstall_command(no_confirm: bool) -> Result<ExitCode> {
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
            return Ok(ExitCode::FAILURE);
        }
    };

    uninstall().await?;

    Ok(ExitCode::SUCCESS)
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
    use eyre::bail;
    use tracing::error;

    let exe_path = std::env::current_exe()?;
    let Some(exe_name) = exe_path.file_name().and_then(|s| s.to_str()) else {
        bail!("Failed to get name of current executable: {exe_path:?}")
    };
    let Some(exe_parent) = exe_path.parent() else {
        bail!("Failed to get parent of current executable: {exe_path:?}")
    };
    let local_bin = fig_util::directories::home_local_bin()?;

    if exe_parent != local_bin {
        bail!(
            "Uninstall is only supported for binaries installed in {local_bin:?}, the current executable is in {exe_parent:?}"
        );
    }

    if exe_name != CLI_BINARY_NAME {
        bail!("Uninstall is only supported for {CLI_BINARY_NAME:?}, the current executable is {exe_name:?}");
    }

    if let Err(err) = auth::logout().await {
        error!(%err, "Failed to logout");
    }
    fig_install::uninstall(fig_install::InstallComponents::all_linux()).await?;
    Ok(())
}

#[cfg(all(unix, not(any(target_os = "macos", target_os = "linux"))))]
async fn uninstall() -> Result<()> {
    eyre::bail!("Guided uninstallation is not supported on this platform. Please uninstall manually.");
}

// #[cfg(target_os = "linux")]
// mod linux {
//     use eyre::Result;
//
//     pub async fn uninstall_apt(pkg: String) -> Result<()> {
//         tokio::process::Command::new("apt")
//             .arg("remove")
//             .arg("-y")
//             .arg(pkg)
//             .status()
//             .await?;
//         std::fs::remove_file("/etc/apt/sources.list.d/fig.list")?;
//         std::fs::remove_file("/etc/apt/keyrings/fig.gpg")?;
//
//         Ok(())
//     }
//
//     pub async fn uninstall_dnf(pkg: String) -> Result<()> {
//         tokio::process::Command::new("dnf")
//             .arg("remove")
//             .arg("-y")
//             .arg(pkg)
//             .status()
//             .await?;
//         std::fs::remove_file("/etc/yum.repos.d/fig.repo")?;
//
//         Ok(())
//     }
//
//     pub async fn uninstall_pacman(pkg: String) -> Result<()> {
//         tokio::process::Command::new("pacman")
//             .arg("-Rs")
//             .arg(pkg)
//             .status()
//             .await?;
//
//         Ok(())
//     }
// }
