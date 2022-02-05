//! Installation, uninstallation, and update of the CLI.

use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use anyhow::{Context, Result};
use crossterm::style::Stylize;
use regex::Regex;
use self_update::update::UpdateStatus;

use crate::{cli::util::dialoguer_theme, daemon, ipc::command::update_command, util::shell::Shell};

bitflags::bitflags! {
    /// The different components that can be installed.
    pub struct InstallComponents: usize {
        const DAEMON   = 0b00000001;
        const DOTFILES = 0b00000010;
        const BINARY   = 0b00000100;
    }
}

pub fn install_cli(install_compoenents: InstallComponents) -> Result<()> {
    if install_compoenents.contains(InstallComponents::DAEMON) {
        install_daemon()?;
    }

    if install_compoenents.contains(InstallComponents::DOTFILES) {
        match dialoguer::Confirm::with_theme(&dialoguer_theme())
        .with_prompt("Do you want dotfiles to modify your shell config (you will have to manually do this otherwise)?")
        .interact()?
        {
            true => {
                install_dotfiles().context("Could not install dotfiles")?;
            }
            false => {
                println!();
                println!("To install dotfiles you will have to add the following to your rc files");
                println!();
                println!(
                    "At the top of your ~/.bashrc or ~/.zshrc or ~/.config/fish/config.fish file:"
                );
                println!("bashrc:    eval \"$(dotfiles shell bash pre)\"");
                println!("zshrc:     eval \"$(dotfiles shell zsh pre)\"");
                println!("fish:      eval \"$(dotfiles shell fish pre)\"");
                println!();
                println!(
                    "At the bottom of your ~/.bashrc or ~/.zshrc or ~/.config/fish/config.fish file:"
                );
                println!("bashrc:    eval \"$(dotfiles shell bash post)\"");
                println!("zshrc:     eval \"$(dotfiles shell zsh post)\"");
                println!("fish:      eval \"$(dotfiles shell fish post)\"");
                println!();
            }
        }
    }

    Ok(())
}

fn install_daemon() -> Result<()> {
    #[cfg(target_os = "macos")]
    daemon::LaunchService::launchd()?.install()?;
    #[cfg(target_os = "linux")]
    daemon::LaunchService::systemd()?.install()?;
    #[cfg(target_os = "windows")]
    return Err(anyhow::anyhow!("Windows is not yet supported"));
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    return Err(anyhow::anyhow!("Unsupported platform"));

    Ok(())
}

fn install_dotfiles() -> Result<()> {
    for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
        if let Ok(path) = shell.get_config_path() {
            if path.exists() {
                // Prepend and append the dotfiles
                let mut file = File::open(&path)?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;

                let mut modified = false;
                let mut lines = vec![];

                let pre_eval = match shell {
                    Shell::Bash => "eval \"$(dotfiles init bash pre)\"",
                    Shell::Zsh => "eval \"$(dotfiles init zsh pre)\"",
                    Shell::Fish => "eval (dotfiles init fish pre)",
                };

                if !contents.contains(pre_eval) {
                    lines.push("# Pre dotfiles eval");
                    lines.push(pre_eval);
                    lines.push("");

                    modified = true;
                }

                lines.extend(contents.lines());

                let post_eval = match shell {
                    Shell::Bash => "eval \"$(dotfiles init bash post)\"",
                    Shell::Zsh => "eval \"$(dotfiles init zsh post)\"",
                    Shell::Fish => "eval (dotfiles init fish post)",
                };

                if !contents.contains(post_eval) {
                    lines.push("");
                    lines.push("# Post dotfiles eval");
                    lines.push(post_eval);
                    lines.push("");

                    modified = true;
                }

                if modified {
                    let mut file = File::create(&path)?;
                    file.write_all(lines.join("\n").as_bytes())?;
                }
            }
        }
    }

    Ok(())
}

pub fn uninstall_cli(install_compoenents: InstallComponents) -> Result<()> {
    if install_compoenents.contains(InstallComponents::DAEMON) {
        uninstall_daemon()?;
    }

    if install_compoenents.contains(InstallComponents::DOTFILES) {
        // Uninstall dotfiles
        match dialoguer::Confirm::with_theme(&dialoguer_theme())
        .with_prompt("Do you want dotfiles to modify your shell config (you will have to manually do this otherwise)?")
        .interact()?
        {
            true => {
                uninstall_dotfiles().context("Could not uninstall dotfiles")?;
            },
            false => {
                println!();
                println!(
                    "To uninstall dotfiles you will have to remove the following from your rc files"
                );
                println!();
                println!(
                    "At the top of your ~/.bashrc or ~/.zshrc or ~/.config/fish/config.fish file:"
                );
                println!("bashrc:    eval \"$(dotfiles init bash pre)\"");
                println!("zshrc:     eval \"$(dotfiles init zsh pre)\"");
                println!("fish:      eval \"$(dotfiles init fish pre)\"");
                println!();
                println!("At the bottom of your ~/.bashrc or ~/.zshrc or ~/.config/fish/config.fish file:");
                println!("bashrc:    eval \"$(dotfiles init bash post)\"");
                println!("zshrc:     eval \"$(dotfiles init zsh post)\"");
                println!("fish:      eval \"$(dotfiles init fish post)\"");
                println!();
            },
        }
    }

    if install_compoenents.contains(InstallComponents::BINARY) {
        // Delete the binary
        let binary_path = Path::new("/usr/local/bin/dotfiles");

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

fn uninstall_dotfiles() -> Result<()> {
    for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
        if let Ok(path) = shell.get_config_path() {
            if path.exists() {
                // Prepend and append the dotfiles
                let mut file = File::open(&path)?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;

                let pre_eval = match shell {
                    Shell::Bash => Regex::new(
                        r#"(?:# Pre dotfiles eval\n)?eval "\$\(dotfiles init bash pre\)"\n{0,2}"#,
                    ),
                    Shell::Zsh => Regex::new(
                        r#"(?:# Pre dotfiles eval\n)?eval "\$\(dotfiles init zsh pre\)"\n{0,2}"#,
                    ),
                    Shell::Fish => Regex::new(
                        r#"(?:# Pre dotfiles eval\n)?eval \(dotfiles init fish pre\)\n{0,2}"#,
                    ),
                }
                .unwrap();

                let contents = pre_eval.replace_all(&contents, "");

                let post_eval_regex = match shell {
                    Shell::Bash => Regex::new(
                        r#"(?:# Post dotfiles eval\n)?eval "\$\(dotfiles init bash post\)"\n{0,2}"#,
                    ),
                    Shell::Zsh => Regex::new(
                        r#"(?:# Post dotfiles eval\n)?eval "\$\(dotfiles init zsh post\)"\n{0,2}"#,
                    ),
                    Shell::Fish => Regex::new(
                        r#"(?:# Post dotfiles eval\n)?eval \(dotfiles init fish post\)\n{0,2}"#,
                    ),
                }
                .unwrap();

                let contents = post_eval_regex.replace_all(&contents, "");

                let mut file = File::create(&path)?;
                file.write_all(contents.as_bytes())?;
            }
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

/// Self-update the dotfiles binary
/// Update will exit the binary if the update was successful
pub async fn update(update_type: UpdateType) -> Result<UpdateStatus> {
    // Let desktop app handle updates on macOS
    #[cfg(target_os = "macos")]
    {
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

            permission_guard()?;

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
