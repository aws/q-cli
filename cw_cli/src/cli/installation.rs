//! Installation, uninstallation, and update of the CLI.

use crossterm::style::Stylize;
use eyre::Result;
use fig_install::{
    install,
    InstallComponents,
};
use fig_util::CODEWHISPERER_CLI_BINARY_NAME;

use crate::util::dialoguer_theme;

#[cfg_attr(windows, allow(unused_variables))]
pub async fn install_cli(install_components: InstallComponents, no_confirm: bool, force: bool) -> Result<()> {
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
                std::process::exit(1);
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

            !dialoguer::Confirm::with_theme(&dialoguer_theme())
                .with_prompt(format!(
                    "Do you want {} to modify your shell config (you will have to manually do this otherwise)?",
                    CODEWHISPERER_CLI_BINARY_NAME
                ))
                .interact()?
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
            println!("To install CodeWhisperer manually you will have to add the following to your rc files");
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
                println!("Could not install files needed for CodeWhisperer:");
                println!("{err}");
            }
        }
    }

    if install_components.contains(InstallComponents::INPUT_METHOD) {
        cfg_if::cfg_if! {
            if #[cfg(target_os = "macos")] {
                if !dialoguer::console::user_attended() {
                    eyre::bail!("You must run with --no-confirm if unattended");
                }

                println!();
                println!("For CodeWhisperer to support some terminals like Kitty, Alacritty, and Wezterm");
                println!("you must enable our Input Method integration.");
                println!();
                println!("To enable the integration, select \"yes\" below and then click Ok in the popup.");
                println!();

                if dialoguer::Select::with_theme(&dialoguer_theme())
                    .with_prompt("Do you want to enable support for input method backed terminals?")
                    .default(0)
                    .items(&["Yes", "No"])
                    .interact_opt()? == Some(0) {
                    install(InstallComponents::INPUT_METHOD).await?;
                }
            }
        }
    }

    Ok(())
}
