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

use crate::{
    cli::util::{dialoguer_theme, permission_guard},
    util::shell::Shell,
};

pub fn install_cli() -> Result<()> {
    permission_guard()?;

    // Install daemons
    #[cfg(target_os = "macos")]
    install_daemon_macos().context("Could not install macOS daemon")?;
    #[cfg(target_os = "linux")]
    install_daemon_linux().context("Could not install systemd daemon")?;
    #[cfg(target_os = "windows")]
    install_daemon_windows().context("Could not install Windows daemon")?;
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    unimplemented!();

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

    Ok(())
}

#[cfg(target_os = "macos")]
fn install_daemon_macos() -> Result<()> {
    use std::process::Command;

    use crate::daemon;

    // Put the daemon plist in /Library/LaunchDaemons
    let plist = daemon::launchd_plist();
    plist
        .write_to_file()
        .with_context(|| format!("Could not write to {}", plist.path.display()))?;

    // Start the daemon using launchctl
    Command::new("launchctl")
        .arg("load")
        .arg(plist.path)
        .output()
        .with_context(|| format!("Could not load {}", plist.path.display()))?;

    Ok(())
}

#[cfg(target_os = "linux")]
fn install_daemon_linux() -> Result<()> {
    use std::process::Command;

    use crate::daemon::{self, get_init_system, InitSystem};

    match get_init_system()? {
        InitSystem::Systemd => {
            // Put the daemon service in /usr/lib/systemd/system
            let service = daemon::systemd_service();
            service
                .write_to_file()
                .with_context(|| format!("Could not write to {}", service.path.display()))?;

            // Enable the daemon using systemctl
            Command::new("systemctl")
                .arg("--now")
                .arg("enable")
                .arg(service.path)
                .output()
                .with_context(|| format!("Could not enable {}", service.path.display()))?;
        }
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn install_daemon_windows() -> Result<()> {
    // Put the daemon service in %APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup
    // let service = include_str!("daemon_files/dotfiles-daemon.bat");
    // let service_path = r"%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup\dotfiles-daemon.bat";
    // std::fs::write(service_path, service)
    //     .with_context(|| format!("Could not write to {}", service_path))?;

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
                    Shell::Bash => "eval \"$(dotfiles shell bash pre)\"",
                    Shell::Zsh => "eval \"$(dotfiles shell zsh pre)\"",
                    Shell::Fish => "eval (dotfiles shell fish pre)",
                };

                if !contents.contains(pre_eval) {
                    lines.push("# Pre dotfiles eval");
                    lines.push(pre_eval);
                    lines.push("");

                    modified = true;
                }

                lines.extend(contents.lines());

                let post_eval = match shell {
                    Shell::Bash => "eval \"$(dotfiles shell bash post)\"",
                    Shell::Zsh => "eval \"$(dotfiles shell zsh post)\"",
                    Shell::Fish => "eval (dotfiles shell fish post)",
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
                        r#"(?:# Pre dotfiles eval\n)?eval "\$\(dotfiles shell bash pre\)"\n{0,2}"#,
                    ),
                    Shell::Zsh => Regex::new(
                        r#"(?:# Pre dotfiles eval\n)?eval "\$\(dotfiles shell zsh pre\)"\n{0,2}"#,
                    ),
                    Shell::Fish => Regex::new(
                        r#"(?:# Pre dotfiles eval\n)?eval \(dotfiles shell fish pre\)\n{0,2}"#,
                    ),
                }
                .unwrap();

                let contents = pre_eval.replace_all(&contents, "");

                let post_eval_regex = match shell {
                    Shell::Bash => Regex::new(
                        r#"(?:# Post dotfiles eval\n)?eval "\$\(dotfiles shell bash post\)"\n{0,2}"#,
                    ),
                    Shell::Zsh => Regex::new(
                        r#"(?:# Post dotfiles eval\n)?eval "\$\(dotfiles shell zsh post\)"\n{0,2}"#,
                    ),
                    Shell::Fish => Regex::new(
                        r#"(?:# Post dotfiles eval\n)?eval \(dotfiles shell fish post\)\n{0,2}"#,
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

/// Uninstall dotfiles
pub fn uninstall_cli() -> Result<()> {
    permission_guard()?;

    // Uninstall daemons
    #[cfg(target_os = "macos")]
    uninstall_daemon_macos()?;
    #[cfg(target_os = "linux")]
    uninstall_daemon_linux()?;
    #[cfg(target_os = "windows")]
    uninstall_daemon_windows()?;
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    unimplemented!();

    // Uninstall dotfiles
    match dialoguer::Confirm::with_theme(&dialoguer_theme())
        .with_prompt("Do you want dotfiles to modify your shell config (you will have to manually do this otherwise)?")
        .interact()? {
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
                println!("bashrc:    eval \"$(dotfiles shell bash pre)\"");
                println!("zshrc:     eval \"$(dotfiles shell zsh pre)\"");
                println!("fish:      eval \"$(dotfiles shell fish pre)\"");
                println!();
                println!("At the bottom of your ~/.bashrc or ~/.zshrc or ~/.config/fish/config.fish file:");
                println!("bashrc:    eval \"$(dotfiles shell bash post)\"");
                println!("zshrc:     eval \"$(dotfiles shell zsh post)\"");
                println!("fish:      eval \"$(dotfiles shell fish post)\"");
                println!();
            },
    }

    // Delete the binary
    let binary_path = Path::new("/usr/local/bin/dotfiles");

    if binary_path.exists() {
        std::fs::remove_file(binary_path)
            .with_context(|| format!("Could not delete {}", binary_path.display()))?;
    }

    println!("\n{}\n", "Dotfiles has been uninstalled".bold());

    Ok(())
}

#[cfg(target_os = "macos")]
fn uninstall_daemon_macos() -> Result<()> {
    use std::process::Command;

    // Stop the daemon using launchctl
    Command::new("launchctl")
        .arg("unload")
        .arg("/Library/LaunchDaemons/io.fig.dotfiles-daemon.plist")
        .output()
        .with_context(|| "Could not unload io.fig.dotfiles-daemon.plist")?;

    // Delete the daemon plist
    let plist_path = Path::new("/Library/LaunchDaemons/io.fig.dotfiles-daemon.plist");

    if plist_path.exists() {
        std::fs::remove_file(plist_path)
            .with_context(|| format!("Could not delete {}", plist_path.display()))?;
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn uninstall_daemon_linux() -> Result<()> {
    use std::process::Command;

    // Disable the daemon using systemctl
    Command::new("systemctl")
        .arg("disable")
        .arg("/usr/lib/systemd/system/dotfiles-daemon.service")
        .output()
        .with_context(|| "Could not disable dotfiles-daemon.service")?;

    // Delete the daemon service
    let service_path = Path::new("/etc/systemd/system/dotfiles-daemon.service");

    if service_path.exists() {
        std::fs::remove_file(service_path)
            .with_context(|| format!("Could not delete {}", service_path.display()))?;
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn uninstall_daemon_windows() -> Result<()> {
    // Delete the daemon service
    let service_path = Path::new(
        "C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs\\Startup\\dotfiles-daemon.exe",
    );

    if service_path.exists() {
        std::fs::remove_file(service_path)
            .with_context(|| format!("Could not delete {}", service_path.display()))?;
    }

    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub enum UpdateType {
    Confirm,
    NoConfirm,
    NoProgress,
}

/// Self-update the dotfiles binary
/// Update will exit the binary if the update was successful
pub fn update(update_type: UpdateType) -> Result<UpdateStatus> {
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
}

pub fn update_cli(no_confirm: bool) -> Result<()> {
    update(if no_confirm {
        UpdateType::NoConfirm
    } else {
        UpdateType::Confirm
    })?;

    Ok(())
}
