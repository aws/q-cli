use clap::Subcommand;
use eyre::Result;
use fig_daemon::Daemon;
use fig_integrations::shell::ShellExt;
use fig_integrations::ssh::SshIntegration;
use fig_integrations::Integration as _;
use fig_util::Shell;
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
    Reinstall {
        /// Integration to reinstall
        #[command(subcommand)]
        integration: Integration,
        /// Suppress status messages
        #[arg(long, short)]
        silent: bool,
    },
    Status {
        /// Integration to check status of
        #[command(subcommand)]
        integration: Integration,
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
    InputMethod,
    #[command(name = "vscode")]
    VSCode,
    #[command(name = "intellij", alias = "jetbrains")]
    IntelliJ,
    #[doc(hidden)]
    All,
}

impl IntegrationsSubcommands {
    pub async fn execute(self) -> Result<()> {
        match self {
            IntegrationsSubcommands::Install { integration, silent } => {
                if let Integration::All = integration {
                    install(Integration::Dotfiles { shell: None }, silent).await?;
                    install(Integration::Daemon, silent).await?;
                    install(Integration::Ssh, silent).await?;
                    #[cfg(target_os = "macos")]
                    install(Integration::InputMethod, silent).await?;
                } else {
                    install(integration, silent).await?;
                }
                Ok(())
            },
            IntegrationsSubcommands::Uninstall { integration, silent } => {
                if let Integration::All = integration {
                    uninstall(Integration::Dotfiles { shell: None }, silent).await?;
                    uninstall(Integration::Daemon, silent).await?;
                    uninstall(Integration::Ssh, silent).await?;
                    #[cfg(target_os = "macos")]
                    uninstall(Integration::InputMethod, silent).await?;
                } else {
                    uninstall(integration, silent).await?;
                }
                Ok(())
            },
            IntegrationsSubcommands::Status { integration } => status(integration).await,
            IntegrationsSubcommands::Reinstall { integration, silent } => {
                if let Integration::All = integration {
                    uninstall(Integration::Dotfiles { shell: None }, silent).await?;
                    uninstall(Integration::Daemon, silent).await?;
                    uninstall(Integration::Ssh, silent).await?;
                    #[cfg(target_os = "macos")]
                    uninstall(Integration::InputMethod, silent).await?;
                    install(Integration::Dotfiles { shell: None }, silent).await?;
                    install(Integration::Daemon, silent).await?;
                    install(Integration::Ssh, silent).await?;
                    #[cfg(target_os = "macos")]
                    install(Integration::InputMethod, silent).await?;
                } else {
                    uninstall(integration, silent).await?;
                    install(integration, silent).await?;
                }
                Ok(())
            },
        }
    }
}

#[allow(unused_mut)]
async fn install(integration: Integration, silent: bool) -> Result<()> {
    let mut installed = false;
    let mut status: Option<&str> = None;

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
                            match integration.is_installed().await {
                                Ok(_) => {
                                    debug!("Skipping {}", integration.describe());
                                },
                                Err(_) => {
                                    installed = true;
                                    if let Err(e) = integration.install().await {
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
            let path = std::env::current_exe()?;
            Daemon::default().install(&path).await?;
            installed = true;
            Ok(())
        },
        Integration::Ssh => {
            let ssh_integration = SshIntegration::new()?;
            if ssh_integration.is_installed().await.is_err() {
                installed = true;
                ssh_integration.install().await.map_err(eyre::Report::from)
            } else {
                Ok(())
            }
        },
        Integration::InputMethod => {
            cfg_if::cfg_if! {
                if #[cfg(target_os = "macos")] {
                    fig_integrations::input_method::InputMethod::default().install().await?;
                    installed = true;
                    status = Some("You must restart your terminal to finish installing the input method.");
                    Ok(())
                } else {
                    Err(eyre::eyre!("Input method integration is only supported on macOS"))
                }
            }
        },
        Integration::VSCode => {
            cfg_if::cfg_if! {
                if #[cfg(target_os = "macos")] {
                    let variants = fig_integrations::vscode::variants_installed();
                    installed = !variants.is_empty();
                    for variant in variants {
                        fig_integrations::vscode::VSCodeIntegration { variant }.install().await?;
                    }
                    Ok(())
                } else {
                    Err(eyre::eyre!("VSCode integration is only supported on macOS"))
                }
            }
        },
        Integration::IntelliJ => {
            cfg_if::cfg_if! {
                if #[cfg(any(target_os = "macos", target_os = "linux"))] {
                    let variants = fig_integrations::intellij::variants_installed().await?;
                    installed = !variants.is_empty();
                    for variant in variants {
                        variant.install().await?;
                    }
                    Ok(())
                } else {
                    Err(eyre::eyre!("IntelliJ integration is only supported on macOS and Linux"))
                }
            }
        },
    };

    if installed && result.is_ok() && !silent {
        println!("Installed!");

        if let Some(status) = status {
            println!("{}", status);
        }
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
                            match integration.is_installed().await {
                                Ok(_) => {
                                    uninstalled = true;
                                    if let Err(e) = integration.uninstall().await {
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
            let ssh_integration = SshIntegration::new()?;
            if ssh_integration.is_installed().await.is_ok() {
                uninstalled = true;
                ssh_integration.uninstall().await.map_err(eyre::Report::from)
            } else {
                Ok(())
            }
        },
        Integration::InputMethod => {
            cfg_if::cfg_if! {
                if #[cfg(target_os = "macos")] {
                    fig_integrations::input_method::InputMethod::default().uninstall().await?;
                    uninstalled = true;
                    Ok(())
                } else {
                    Err(eyre::eyre!("Input method integration is only supported on macOS"))
                }
            }
        },
        Integration::VSCode => {
            cfg_if::cfg_if! {
                if #[cfg(target_os = "macos")] {
                    for variant in fig_integrations::vscode::variants_installed() {
                        let integration = fig_integrations::vscode::VSCodeIntegration { variant };
                        if integration.is_installed().await.is_ok() {
                            integration.uninstall().await?;
                            uninstalled = true;
                        }
                    }
                    println!("Warning: VSCode integrations are automatically reinstalled on launch");
                    Ok(())
                } else {
                    Err(eyre::eyre!("VSCode integration is only supported on macOS"))
                }
            }
        },
        Integration::IntelliJ => {
            cfg_if::cfg_if! {
                if #[cfg(any(target_os = "macos", target_os = "linux"))] {
                    for variant in fig_integrations::intellij::variants_installed().await? {
                        if variant.is_installed().await.is_ok() {
                            variant.uninstall().await?;
                            uninstalled = true;
                        }
                    }
                    println!("Warning: IntelliJ integrations are automatically reinstalled on launch");
                    Ok(())
                } else {
                    Err(eyre::eyre!("IntelliJ integration is only supported on macOS and Linux"))
                }
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

async fn status(integration: Integration) -> Result<()> {
    match integration {
        Integration::All => Err(eyre::eyre!("Cannot check status of all integrations")),
        Integration::Ssh => {
            let ssh_integration = SshIntegration::new()?;
            if ssh_integration.is_installed().await.is_ok() {
                println!("Installed")
            } else {
                println!("Not installed")
            }
            Ok(())
        },
        Integration::Daemon => {
            let daemon = Daemon::default();
            match daemon.status().await {
                Ok(status) => {
                    println!("Status: {status:?}");
                },
                Err(err) => {
                    println!("Status Error: {err}");
                },
            }
            Ok(())
        },
        Integration::Dotfiles { .. } => {
            for shell in &[Shell::Bash, Shell::Zsh, Shell::Fish] {
                match shell.get_shell_integrations() {
                    Ok(integrations) => {
                        for integration in integrations {
                            match integration.is_installed().await {
                                Ok(_) => {
                                    println!("{}: Installed", integration.describe());
                                },
                                Err(_) => {
                                    println!("{}: Not installed", integration.describe());
                                },
                            }
                        }
                    },
                    Err(e) => {
                        println!("{shell}: {e}");
                    },
                }
            }
            Ok(())
        },
        Integration::InputMethod => {
            cfg_if::cfg_if! {
                if #[cfg(target_os = "macos")] {
                    let input_method = fig_integrations::input_method::InputMethod::default();
                    if input_method.is_installed().await.is_ok() {
                        println!("Installed")
                    } else {
                        println!("Not installed")
                    }
                    Ok(())
                } else {
                    Err(eyre::eyre!("Input method integration is only supported on macOS"))
                }
            }
        },
        Integration::VSCode => {
            cfg_if::cfg_if! {
                if #[cfg(target_os = "macos")] {
                    let variants = fig_integrations::vscode::variants_installed();
                    for variant in variants {
                        let integration = fig_integrations::vscode::VSCodeIntegration { variant };
                        match integration.is_installed().await {
                            Ok(_) => {
                                println!("{}: Installed", integration.variant.application_name);
                            }
                            Err(_) => {
                                println!("{}: Not installed", integration.variant.application_name);
                            }
                        }
                    }
                    Ok(())
                } else {
                    Err(eyre::eyre!("VSCode integration is only supported on macOS"))
                }
            }
        },
        Integration::IntelliJ => {
            cfg_if::cfg_if! {
                if #[cfg(any(target_os = "macos", target_os = "linux"))] {
                    let variants = fig_integrations::intellij::variants_installed().await?;
                    for variant in variants {
                        match variant.is_installed().await {
                            Ok(_) => {
                                println!("{}: Installed", variant.variant.application_name());
                            }
                            Err(_) => {
                                println!("{}: Not installed", variant.variant.application_name());
                            }
                        }
                    }
                    Ok(())
                } else {
                    Err(eyre::eyre!("IntelliJ integration is only supported on macOS and Linux"))
                }
            }
        },
    }
}
