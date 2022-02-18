//! Installation, uninstallation, and update of the CLI.

use std::path::Path;

use anyhow::{Context, Result};
use crossterm::style::Stylize;
use self_update::update::UpdateStatus;
use time::OffsetDateTime;

use crate::{
    cli::util::dialoguer_theme,
    daemon,
    util::{home_dir, shell::Shell},
};

bitflags::bitflags! {
    /// The different components that can be installed.
    pub struct InstallComponents: usize {
        const DAEMON   = 0b00000001;
        const DOTFILES = 0b00000010;
        const BINARY   = 0b00000100;
    }
}

pub fn install_cli(install_components: InstallComponents) -> Result<()> {
    if install_components.contains(InstallComponents::DAEMON) {
        daemon::install_daemon()?;
    }

    if install_components.contains(InstallComponents::DOTFILES) {
        let mut manual_install = !dialoguer::Confirm::with_theme(&dialoguer_theme())
        .with_prompt("Do you want fig to modify your shell config (you will have to manually do this otherwise)?")
        .interact()?;
        if !manual_install {
            if let Err(e) = install_fig() {
                println!("Could not automatically install: {}", e);
                manual_install = true;
            }
        }
        if manual_install {
            println!();
            println!("To install fig manually you will have to add the following to your rc files");
            println!();
            println!(
                "At the top of your .bashrc or .zshrc or .config/fish/conf.d/00_fig_pre.fish file:"
            );
            println!("bash:    eval \"$(fig init bash pre)\"");
            println!("zsh:     eval \"$(fig init zsh pre)\"");
            println!("fish:    eval \"$(fig init fish pre)\"");
            println!();
            println!(
                "At the bottom of your .bashrc or .zshrc or .config/fish/conf.d/99_fig_post.fish file:"
            );
            println!("bash:    eval \"$(fig init bash post)\"");
            println!("zsh:     eval \"$(fig init zsh post)\"");
            println!("fish:    eval \"$(fig init fish post)\"");
            println!();
        }
    }

    Ok(())
}

fn install_fig() -> Result<()> {
    let now = OffsetDateTime::now_utc().format(time::macros::format_description!(
        "[year]-[month]-[day]_[hour]-[minute]-[second]"
    ))?;
    let backup_dir = home_dir()?.join(".fig.dotfiles.bak").join(now);
    for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
        for integration in shell.get_shell_integrations()? {
            integration.install(Some(&backup_dir))?
        }
    }

    Ok(())
}

pub fn uninstall_cli(install_components: InstallComponents) -> Result<()> {
    if install_components.contains(InstallComponents::DAEMON) {
        uninstall_daemon()?;
    }

    if install_components.contains(InstallComponents::DOTFILES) {
        // Uninstall fig
        let mut manual_uninstall = !dialoguer::Confirm::with_theme(&dialoguer_theme())
        .with_prompt("Do you want fig to modify your shell config (you will have to manually do this otherwise)?")
        .interact()?;
        if !manual_uninstall && uninstall_fig().is_err() {
            println!("Could not uninstall fig");
            manual_uninstall = true;
        }
        if manual_uninstall {
            println!();
            println!("To uninstall fig you should follow the instructions for your shell(s):");
            println!();
            println!("{}", "bash".bold().underlined());
            println!(
                "1. Remove {} from the top of your .bashrc, .bash_profile, .bash_login, and/or .profile files", "eval \"$(fig init bash pre)\"".magenta()
            );
            println!(
                "2. Remove {} from the bottom of your .bashrc, .bash_profile, .bash_login, and/or .profile files", "eval \"$(fig init bash post)\"".magenta()
            );
            println!();

            println!("{}", "zsh".bold().underlined());
            println!(
                "1. Remove {} from the top of your .zshrc and/or .zprofile",
                "eval \"$(fig init zsh pre)\"".magenta()
            );
            println!(
                "2. Remove {} from the bottom of your .zshrc, and/or .zprofile files",
                "eval \"$(fig init zsh post)\"".magenta()
            );
            println!();

            println!("{}", "fish".bold().underlined());
            println!("Remove the 00_fig_pre.fish and 99_fig_post.fish files from your .config/fish/conf.d directory.");
            // Print instructions for manual installation.
            println!();
        }
    }

    if install_components.contains(InstallComponents::BINARY) {
        // Delete the binary
        let binary_path = Path::new("/usr/local/bin/fig");

        if binary_path.exists() {
            std::fs::remove_file(binary_path)
                .with_context(|| format!("Could not delete {}", binary_path.display()))?;
        }

        println!("\n{}\n", "Dotfiles has been uninstalled".bold());
    }

    Ok(())
}

fn uninstall_daemon() -> Result<()> {
    #[cfg(target_os = "macos")]
    daemon::LaunchService::launchd()?.uninstall()?;
    #[cfg(target_os = "linux")]
    daemon::LaunchService::systemd()?.uninstall()?;
    #[cfg(target_os = "windows")]
    return Err(anyhow::anyhow!("Windows is not yet supported"));
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    return Err(anyhow::anyhow!("Unsupported platform"));

    Ok(())
}

fn uninstall_fig() -> Result<()> {
    for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
        for integration in shell.get_shell_integrations()? {
            integration.uninstall()?
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateType {
    Confirm,
    NoConfirm,
    NoProgress,
}

/// Self-update the fig binary
/// Update will exit the binary if the update was successful
pub async fn update(update_type: UpdateType) -> Result<UpdateStatus> {
    // Let desktop app handle updates on macOS
    #[cfg(target_os = "macos")]
    {
        use fig_ipc::command::update_command;

        let desktop_app_update = update_command(update_type == UpdateType::Confirm).await;
        match desktop_app_update {
            Ok(()) => {
                println!("\n→ Checking for updates to macOS app...\n");
                Ok(UpdateStatus::UpToDate)
            }
            Err(_) => {
                anyhow::bail!(
                    "\n{}\nFig might not be running to launch Fig run: {}\n",
                    "Unable to Connect to Fig:".bold(),
                    "fig launch".magenta()
                );
            }
        }
    }

    #[cfg(not(any(target_os = "macos")))]
    {
        let confirm = match update_type {
            UpdateType::Confirm => true,
            UpdateType::NoConfirm => false,
            UpdateType::NoProgress => false,
        };

        let progress_output = match update_type {
            UpdateType::Confirm => true,
            UpdateType::NoConfirm => true,
            UpdateType::NoProgress => false,
        };

        tokio::task::block_in_place(move || {
            let current_version = env!("CARGO_PKG_VERSION");

            let update = self_update::backends::s3::Update::configure()
                .bucket_name("get-fig-io")
                .asset_prefix("bin")
                .region("us-west-1")
                .bin_name("dotfiles")
                .current_version(current_version)
                .no_confirm(true)
                .show_output(false)
                .show_download_progress(progress_output)
                .build()?;

            let latest_release = update.get_latest_release()?;

            if !self_update::version::bump_is_greater(current_version, &latest_release.version)? {
                println!("You are already on the latest version {}", current_version);

                return Ok(UpdateStatus::UpToDate);
            }

            let confirm = match update_type {
                UpdateType::Confirm => true,
                UpdateType::NoConfirm => false,
                UpdateType::NoProgress => false,
            };

            let progress_output = match update_type {
                UpdateType::Confirm => true,
                UpdateType::NoConfirm => true,
                UpdateType::NoProgress => false,
            };

            tokio::task::block_in_place(move || {
                let current_version = env!("CARGO_PKG_VERSION");

                let update = self_update::backends::s3::Update::configure()
                    .bucket_name("get-fig-io")
                    .asset_prefix("bin")
                    .region("us-west-1")
                    .bin_name("dotfiles")
                    .current_version(current_version)
                    .no_confirm(true)
                    .show_output(false)
                    .show_download_progress(progress_output)
                    .build()?;

                let latest_release = update.get_latest_release()?;

                if !self_update::version::bump_is_greater(current_version, &latest_release.version)?
                {
                    println!("You are already on the latest version {}", current_version);

                    return Ok(UpdateStatus::UpToDate);
                }

                if confirm {
                    if !dialoguer::Confirm::with_theme(&dialoguer_theme())
                        .with_prompt(format!(
                            "Do you want to update {} from {} to {}?",
                            env!("CARGO_PKG_NAME"),
                            update.current_version(),
                            latest_release.version
                        ))
                        .default(true)
                        .interact()?
                    {
                        return Err(anyhow::anyhow!("Update cancelled"));
                    }
                } else {
                    println!(
                        "Updating {} from {} to {}",
                        env!("CARGO_PKG_NAME"),
                        update.current_version(),
                        latest_release.version
                    );
                }

                Ok(update.update_extended()?)
            })
        })
    }
}

pub async fn update_cli(no_confirm: bool) -> Result<()> {
    update(if no_confirm {
        UpdateType::NoConfirm
    } else {
        UpdateType::Confirm
    })
    .await?;

    Ok(())
}
