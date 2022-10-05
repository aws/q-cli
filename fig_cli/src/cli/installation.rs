//! Installation, uninstallation, and update of the CLI.

use std::convert::TryInto;
use std::path::{
    Path,
    PathBuf,
};

use crossterm::style::Stylize;
use eyre::{
    Result,
    WrapErr,
};
use fig_daemon::Daemon;
use fig_integrations::shell::ShellExt;
use fig_integrations::ssh::SshIntegration;
use fig_integrations::Integration;
use fig_util::{
    directories,
    Shell,
};

use crate::util::dialoguer_theme;

bitflags::bitflags! {
    /// The different components that can be installed.
    pub struct InstallComponents: usize {
        const DAEMON   = 0b00000001;
        const DOTFILES = 0b00000010;
        const BINARY   = 0b00000100;
        const SSH      = 0b00001000;
    }
}

pub fn get_ssh_config_path() -> Result<PathBuf> {
    Ok(directories::home_dir()
        .context("Could not get home directory")?
        .join(".ssh")
        .join("config"))
}

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

    if install_components.contains(InstallComponents::DOTFILES) {
        let mut manual_install = if no_confirm {
            false
        } else {
            if !dialoguer::console::user_attended() {
                eyre::bail!("You must run with --no-confirm if unattended");
            }

            !dialoguer::Confirm::with_theme(&dialoguer_theme())
                .with_prompt(
                    "Do you want fig to modify your shell config (you will have to manually do this otherwise)?",
                )
                .interact()?
        };
        if !manual_install {
            if let Err(err) = install_fig(true) {
                println!("{}", "Could not automatically install:".bold());
                println!("{err}");
                manual_install = true;
            }
        }
        if !no_confirm && manual_install {
            println!();
            println!("To install Fig manually you will have to add the following to your rc files");
            println!();
            println!("At the top of your .bashrc or .zshrc file:");
            println!("bash:    . \"$HOME/.fig/shell/bashrc.pre.bash\"");
            println!("zsh:     . \"$HOME/.fig/shell/zshrc.pre.zsh\"");
            println!();
            println!("At the bottom of your .bashrc or .zshrc file:");
            println!("bash:    . \"$HOME/.fig/shell/bashrc.post.bash\"");
            println!("zsh:     . \"$HOME/.fig/shell/zshrc.post.zsh\"");
            println!();

            if let Err(err) = install_fig(false) {
                println!("Could not install files needed for Fig:");
                println!("{err}");
            }
        }
    }

    // TODO: (mia)
    // Disable ssh by default for now.
    // if install_components.contains(InstallComponents::SSH) {
    // let ssh_integration = SshIntegration { path: get_ssh_config_path()? };
    // if let Err(e) = ssh_integration.install(None) {
    // println!("{}\n {}", "Failed to install SSH integration.".bold(), e);
    // }
    // }

    if install_components.contains(InstallComponents::DAEMON) {
        let path: camino::Utf8PathBuf = std::env::current_exe()?.try_into()?;
        Daemon::default().install(&path).await?;
    }

    Ok(())
}

fn install_fig(_modify_files: bool) -> Result<()> {
    let backup_dir = directories::utc_backup_dir()?;

    let mut errs: Vec<String> = vec![];
    for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
        match shell.get_shell_integrations() {
            Ok(integrations) => {
                for integration in integrations {
                    if let Err(e) = integration.install(Some(&backup_dir)) {
                        errs.push(format!("{}: {}", integration, e));
                    }
                }
            },
            Err(e) => {
                errs.push(format!("{}: {}", shell, e));
            },
        }
    }

    if errs.is_empty() {
        Ok(())
    } else {
        Err(eyre::eyre!(errs.join("\n")))
    }
}

pub async fn uninstall_cli(install_components: InstallComponents) -> Result<()> {
    let daemon_result = if install_components.contains(InstallComponents::DAEMON) {
        Daemon::default().uninstall().await?;
        Ok(())
    } else {
        Ok(())
    };

    let dotfiles_result = if install_components.contains(InstallComponents::DOTFILES) {
        uninstall_fig()
    } else {
        Ok(())
    };

    let ssh_result = if install_components.contains(InstallComponents::SSH) {
        let ssh_integration = SshIntegration {
            path: get_ssh_config_path()?,
        };
        ssh_integration.uninstall()
    } else {
        Ok(())
    };

    if install_components.contains(InstallComponents::BINARY) {
        if option_env!("FIG_IS_PACKAGE_MANAGED").is_some() {
            println!("Uninstall Fig via your package manager");
        } else {
            let local_path = directories::home_dir()
                .context("Could not find home directory")?
                .join(".local")
                .join("bin")
                .join("fig");
            let binary_paths = [Path::new("/usr/local/bin/fig"), local_path.as_path()];

            for path in binary_paths {
                if path.exists() {
                    std::fs::remove_file(path).with_context(|| format!("Could not delete {}", path.display()))?;
                }
            }
            println!("\n{}\n", "Fig binary has been uninstalled".bold())
        }
    }

    daemon_result
        .and(dotfiles_result)
        .and(ssh_result.map_err(eyre::Report::from))
}

fn uninstall_fig() -> Result<()> {
    for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
        for integration in shell.get_shell_integrations()? {
            integration.uninstall()?
        }
    }

    Ok(())
}
