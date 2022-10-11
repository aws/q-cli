use std::path::PathBuf;

use clap::Subcommand;
use eyre::{
    Result,
    WrapErr,
};
use fig_daemon::Daemon;
use fig_integrations::shell::ShellExt;
use fig_integrations::ssh::SshIntegration;
use fig_integrations::Integration as _;
use fig_util::{
    directories,
    Shell,
};
use tracing::debug;

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum IntegrationsSubcommands {
    Install {
        /// Integration to install
        #[command(subcommand)]
        integration: Integration,
        /// Suppress status messages
        #[arg(long, short)]
        silent: bool,
    },
    Uninstall {
        /// Integration to uninstall
        #[command(subcommand)]
        integration: Integration,
        /// Suppress status messages
        #[arg(long, short)]
        silent: bool,
    },
}

#[derive(Debug, Subcommand, Clone, Copy, PartialEq, Eq)]
pub enum Integration {
    Dotfiles {
        /// Only install the integrations for a single shell
        #[arg(value_enum)]
        shell: Option<Shell>,
    },
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
                    install(Integration::Dotfiles { shell: None }, silent).await?;
                    install(Integration::Daemon, silent).await?;
                    install(Integration::Ssh, silent).await
                } else {
                    install(integration, silent).await
                }
            },
            IntegrationsSubcommands::Uninstall { integration, silent } => {
                if let Integration::All = integration {
                    uninstall(Integration::Dotfiles { shell: None }, silent).await?;
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
    let backup_dir = directories::utc_backup_dir().context("Could not get backup dir")?;

    let mut installed = false;

    let result = match integration {
        Integration::All => Ok(()),
        Integration::Dotfiles { shell } => {
            let shells = match shell {
                Some(shell) => vec![shell],
                None => vec![Shell::Bash, Shell::Zsh, Shell::Fish],
            };

            let mut errs: Vec<String> = vec![];
            for shell in &shells {
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
            let path: camino::Utf8PathBuf = std::env::current_exe()?.try_into()?;
            Daemon::default().install(&path).await?;
            installed = true;
            Ok(())
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
        Integration::Dotfiles { shell } => {
            let shells = match shell {
                Some(shell) => vec![shell],
                None => vec![Shell::Bash, Shell::Zsh, Shell::Fish],
            };

            let mut errs: Vec<String> = vec![];
            for shell in &shells {
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
            Daemon::default().uninstall().await?;
            uninstalled = true;
            Ok(())
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
