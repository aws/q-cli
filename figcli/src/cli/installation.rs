//! Installation, uninstallation, and update of the CLI.

use std::path::Path;

use anyhow::{Context, Result};
use crossterm::style::Stylize;
use nix::unistd::geteuid;
use self_update::update::UpdateStatus;
use time::OffsetDateTime;

use crate::{cli::util::dialoguer_theme, daemon, util::shell::Shell};

bitflags::bitflags! {
    /// The different components that can be installed.
    pub struct InstallComponents: usize {
        const DAEMON   = 0b00000001;
        const DOTFILES = 0b00000010;
        const BINARY   = 0b00000100;
    }
}

pub fn install_cli(
    install_components: InstallComponents,
    no_confirm: bool,
    force: bool,
) -> Result<()> {
    #[cfg(target_family = "unix")]
    {
        if geteuid().is_root() {
            eprintln!("{}", "Installing as root is not supported.".red().bold());
            if !force {
                eprintln!(
                    "{}",
                    "If you know what you're doing, run the command again with --force.".red()
                );
                std::process::exit(1);
            }
        }
    }

    if install_components.contains(InstallComponents::DOTFILES) {
        let mut manual_install = if no_confirm {
            false
        } else {
            !dialoguer::Confirm::with_theme(&dialoguer_theme())
            .with_prompt("Do you want fig to modify your shell config (you will have to manually do this otherwise)?")
            .interact()?
        };
        if !manual_install {
            if let Err(e) = install_fig() {
                println!("{}\n {}", "Could not automatically install:".bold(), e);
                manual_install = true;
            }
        }
        if !no_confirm && manual_install {
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

    if install_components.contains(InstallComponents::DAEMON) {
        daemon::install_daemon()?;
    }

    Ok(())
}

fn install_fig() -> Result<()> {
    let now = OffsetDateTime::now_utc().format(time::macros::format_description!(
        "[year]-[month]-[day]_[hour]-[minute]-[second]"
    ))?;
    let backup_dir = fig_directories::home_dir()
        .context("Could not find home directory")?
        .join(".fig.dotfiles.bak")
        .join(now);

    let mut errs: Vec<String> = vec![];
    for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
        match shell.get_shell_integrations() {
            Ok(integrations) => {
                for integration in integrations {
                    if let Err(e) = integration.install(Some(&backup_dir)) {
                        errs.push(format!("{}: {}", integration, e));
                    }
                }
            }
            Err(e) => {
                errs.push(format!("{}: {}", shell, e));
            }
        }
    }

    if errs.is_empty() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(errs.join("\n")))
    }
}

pub fn uninstall_cli(install_components: InstallComponents) -> Result<()> {
    let daemon_result = if install_components.contains(InstallComponents::DAEMON) {
        uninstall_daemon()
    } else {
        Ok(())
    };

    let dotfiles_result = if install_components.contains(InstallComponents::DOTFILES) {
        uninstall_fig()
    } else {
        Ok(())
    };

    if install_components.contains(InstallComponents::BINARY) {
        let local_path = fig_directories::home_dir()
            .context("Could not find home directory")?
            .join(".local")
            .join("bin")
            .join("fig");
        let binary_paths = [Path::new("/usr/local/bin/fig"), local_path.as_path()];

        for path in binary_paths {
            if path.exists() {
                std::fs::remove_file(path)
                    .with_context(|| format!("Could not delete {}", path.display()))?;
            }
        }

        println!("\n{}\n", "Fig binary has been uninstalled".bold())
    }

    daemon_result.and(dotfiles_result)
}

fn uninstall_daemon() -> Result<()> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            daemon::LaunchService::launchd()?.uninstall()
        } else if #[cfg(target_os = "linux")] {
            daemon::LaunchService::systemd()?.uninstall()
        } else if #[cfg(target_os = "windows")] {
            Err(anyhow::anyhow!("Windows is not yet supported"))
        } else {
            Err(anyhow::anyhow!("Unsupported platform"))
        }
    }
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
    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            // Let desktop app handle updates on macOS
            use crate::util::{launch_fig, LaunchOptions};
            use fig_ipc::command::update_command;

            launch_fig(LaunchOptions::new().wait_for_activation().verbose())?;

            let desktop_app_update = update_command(update_type == UpdateType::NoConfirm).await;
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
        } else {
            let _confirm = match update_type {
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
                    .bin_name("fig")
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
                        .bin_name("fig")
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
