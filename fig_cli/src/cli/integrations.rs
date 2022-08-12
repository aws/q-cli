use std::path::PathBuf;

use clap::Subcommand;
use eyre::{
    ContextCompat,
    Result,
    WrapErr,
};
use fig_integrations::shell::ShellExt;
use fig_integrations::ssh::SshIntegration;
use fig_integrations::{
    get_default_backup_dir,
    Integration as _,
};
use fig_util::{
    directories,
    Shell,
};
use tracing::debug;

use crate::daemon;

#[derive(Debug, Subcommand)]
pub enum IntegrationsSubcommands {
    Install {
        /// Integration to install
        #[clap(subcommand)]
        integration: Integration,
        /// Suppress status messages
        #[clap(long, short)]
        silent: bool,
    },
    Uninstall {
        /// Integration to uninstall
        #[clap(subcommand)]
        integration: Integration,
        /// Suppress status messages
        #[clap(long, short)]
        silent: bool,
    },
}

#[derive(Debug, Subcommand, Clone, Copy)]
pub enum Integration {
    Dotfiles,
    Daemon,
    Ssh,
    #[doc(hidden)]
    All,
}

pub fn get_ssh_config_path() -> Result<PathBuf> {
    Ok(directories::home_dir()
        .context("Could not get home directory")?
        .join(".ssh")
        .join("config"))
}

impl IntegrationsSubcommands {
    pub async fn execute(self) -> Result<()> {
        match self {
            IntegrationsSubcommands::Install { integration, silent } => {
                if let Integration::All = integration {
                    install(Integration::Dotfiles, silent).await?;
                    install(Integration::Daemon, silent).await?;
                    install(Integration::Ssh, silent).await
                } else {
                    install(integration, silent).await
                }
            },
            IntegrationsSubcommands::Uninstall { integration, silent } => {
                if let Integration::All = integration {
                    uninstall(Integration::Dotfiles, silent).await?;
                    uninstall(Integration::Daemon, silent).await?;
                    uninstall(Integration::Ssh, silent).await
                } else {
                    uninstall(integration, silent).await
                }
            },
        }
    }
}

async fn install(integration: Integration, silent: bool) -> Result<()> {
    let backup_dir = get_default_backup_dir().context("Could not get backup dir")?;

    let mut installed = false;

    let result = match integration {
        Integration::All => Ok(()),
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
                Err(eyre::eyre!(errs.join("\n")))
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
                ssh_integration.install(Some(&backup_dir)).map_err(eyre::Report::from)
            } else {
                Ok(())
            }
        },
    };

    if installed && result.is_ok() && !silent {
        println!("Installed!")
    }

    if !installed && !silent {
        println!("Already installed")
    }

    result
}

async fn uninstall(integration: Integration, silent: bool) -> Result<()> {
    let mut uninstalled = false;

    let result = match integration {
        Integration::All => Ok(()),
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
                Err(eyre::eyre!(errs.join("\n")))
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
                ssh_integration.uninstall().map_err(eyre::Report::from)
            } else {
                Ok(())
            }
        },
    };

    if uninstalled && result.is_ok() && !silent {
        println!("Uninstalled!")
    }

    if !uninstalled && !silent {
        println!("Not installed")
    }

    result
}
