//! Installation, uninstallation, and update of the CLI.

use std::process::ExitCode;

use anstream::{
    eprintln,
    println,
};
use crossterm::style::Stylize;
use eyre::Result;
use fig_install::{
    install,
    InstallComponents,
};
use fig_util::{
    CLI_BINARY_NAME,
    PRODUCT_NAME,
};

use super::user::login_interactive;
use crate::util::choose;

#[cfg_attr(windows, allow(unused_variables))]
pub async fn install_cli(install_components: InstallComponents, no_confirm: bool, force: bool) -> Result<ExitCode> {
    #[cfg(unix)]
    {
        use nix::unistd::geteuid;
        if geteuid().is_root() {
            eprintln!("{}", "Installing as root is not supported.".red().bold());
            if !force {
                eprintln!(
                    "{}",
                    "If you know what you're doing, run the command again with --force.".red()
                );
                return Ok(ExitCode::FAILURE);
            }
        }
    }

    if install_components.contains(InstallComponents::SHELL_INTEGRATIONS) {
        let mut manual_install = if no_confirm {
            false
        } else {
            if !dialoguer::console::user_attended() {
                eyre::bail!("You must run with --no-confirm if unattended");
            }

            choose(
                format!(
                    "Do you want {CLI_BINARY_NAME} to modify your shell config (you will have to manually do this otherwise)?",
                ),
                &["Yes", "No"],
            )? == 1
        };
        if !manual_install {
            if let Err(err) = install(InstallComponents::SHELL_INTEGRATIONS).await {
                println!("{}", "Could not automatically install:".bold());
                println!("{err}");
                manual_install = true;
            }
        }
        if !no_confirm && manual_install {
            let shell_dir = fig_util::directories::fig_data_dir_utf8()?.join("shell");
            let shell_dir = shell_dir
                .strip_prefix(fig_util::directories::home_dir()?)
                .unwrap_or(&shell_dir);

            println!();
            println!("To install the integrations manually, you will have to add the following to your rc files");
            println!("This step is required for the application to function properly");
            println!();
            println!("At the top of your .bashrc or .zshrc file:");
            println!("bash:    . \"$HOME/{shell_dir}/bashrc.pre.bash\"");
            println!("zsh:     . \"$HOME/{shell_dir}/zshrc.pre.zsh\"");
            println!();
            println!("At the bottom of your .bashrc or .zshrc file:");
            println!("bash:    . \"$HOME/{shell_dir}/bashrc.post.bash\"");
            println!("zsh:     . \"$HOME/{shell_dir}/zshrc.post.zsh\"");
            println!();

            if let Err(err) = install(InstallComponents::SHELL_INTEGRATIONS).await {
                println!("Could not install required files:");
                println!("{err}");
            }
        }
    }

    if install_components.contains(InstallComponents::INPUT_METHOD) && !no_confirm {
        cfg_if::cfg_if! {
            if #[cfg(target_os = "macos")] {
                if !dialoguer::console::user_attended() {
                    eyre::bail!("You must run with --no-confirm if unattended");
                }

                println!();
                println!("To enable support for some terminals like Kitty, Alacritty, and Wezterm,");
                println!("you must enable our Input Method integration.");
                println!();
                println!("To enable the integration, select \"yes\" below and then click Ok in the popup.");
                println!();

                if choose("Do you want to enable support for input method backed terminals?", &["Yes", "No"])? == 0 {
                    install(InstallComponents::INPUT_METHOD).await?;
                }
            }
        }
    }

    if !auth::is_logged_in().await {
        if !no_confirm {
            if !dialoguer::console::user_attended() {
                eyre::bail!("You must run with --no-confirm if unattended");
            }

            login_interactive().await?;
        } else {
            println!();
            println!("You must login before you can use {PRODUCT_NAME}'s features.");
            println!("To login run: {}", format!("{CLI_BINARY_NAME} login").bold());
            println!();
        }
    }

    Ok(ExitCode::SUCCESS)
}
