use std::path::PathBuf;

use anyhow::{
    Context,
    Result,
};
use clap::Subcommand;
use fig_integrations::shell::ShellExt;
use fig_integrations::ssh::SshIntegration;
use fig_integrations::{
    get_default_backup_dir,
    Integration as _,
};
use fig_util::Shell;
use tracing::debug;

use crate::daemon;

#[derive(Debug, Subcommand)]
pub enum IntegrationsSubcommands {
    Install {
        // Integration to install
        #[clap(subcommand)]
        integration: Integration,
    },
    Uninstall {
        // Integration to uninstall
        #[clap(subcommand)]
        integration: Integration,
    },
}

#[derive(Debug, Subcommand)]
pub enum Integration {
    Dotfiles,
    Daemon,
    Ssh,
}

pub fn get_ssh_config_path() -> Result<PathBuf> {
    Ok(fig_directories::home_dir()
        .context("Could not get home directory")?
        .join(".ssh")
        .join("config"))
}

impl IntegrationsSubcommands {
    pub async fn execute(self) -> Result<()> {
        match self {
            IntegrationsSubcommands::Install { integration } => install(integration).await,
            IntegrationsSubcommands::Uninstall { integration } => uninstall(integration).await,
        }
    }
}

async fn install(integration: Integration) -> Result<()> {
    let backup_dir = get_default_backup_dir()?;

    let mut installed = false;

    let result = match integration {
        Integration::Dotfiles => {
            let mut errs: Vec<String> = vec![];
            for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
                match shell.get_shell_integrations() {
                    Ok(integrations) => {
                        for integration in integrations {
                            match integration.is_installed() {
                                Ok(_) => {
                                    debug!("Skipping {}", integration.describe());
                                },
                                Err(_) => {
                                    installed = true;
                                    if let Err(e) = integration.install(Some(&backup_dir)) {
                                        errs.push(format!("{}: {e}", integration.describe()));
                                    }
                                },
                            }
                        }
                    },
                    Err(e) => {
                        errs.push(format!("{shell}: {e}"));
                    },
                }
            }

            if errs.is_empty() {
                Ok(())
            } else {
                Err(anyhow::anyhow!(errs.join("\n")))
            }
        },
        Integration::Daemon => {
            installed = true;
            daemon::install_daemon()
        },
        Integration::Ssh => {
            let ssh_integration = SshIntegration {
                path: get_ssh_config_path()?,
            };
            if ssh_integration.is_installed().is_err() {
                installed = true;
                ssh_integration.install(Some(&backup_dir))
            } else {
                Ok(())
            }
        },
    };

    if installed && result.is_ok() {
        println!("Installed!")
    }

    if !installed {
        println!("Already installed")
    }

    result
}

async fn uninstall(integration: Integration) -> Result<()> {
    let mut uninstalled = false;

    let result = match integration {
        Integration::Dotfiles => {
            let mut errs: Vec<String> = vec![];
            for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
                match shell.get_shell_integrations() {
                    Ok(integrations) => {
                        for integration in integrations {
                            match integration.is_installed() {
                                Ok(_) => {
                                    uninstalled = true;
                                    if let Err(e) = integration.uninstall() {
                                        errs.push(format!("{}: {e}", integration.describe()));
                                    }
                                },
                                Err(_) => {
                                    debug!("Skipping {}", integration.describe());
                                },
                            }
                        }
                    },
                    Err(e) => {
                        errs.push(format!("{shell}: {e}"));
                    },
                }
            }

            if errs.is_empty() {
                Ok(())
            } else {
                Err(anyhow::anyhow!(errs.join("\n")))
            }
        },
        Integration::Daemon => {
            uninstalled = true;
            daemon::uninstall_daemon()
        },
        Integration::Ssh => {
            let ssh_integration = SshIntegration {
                path: get_ssh_config_path()?,
            };
            if ssh_integration.is_installed().is_ok() {
                uninstalled = true;
                ssh_integration.uninstall()
            } else {
                Ok(())
            }
        },
    };

    if uninstalled && result.is_ok() {
        println!("Uninstalled!")
    }

    if !uninstalled {
        println!("Not installed")
    }

    result
}
