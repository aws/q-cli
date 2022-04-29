use crate::integrations::{ssh::SshIntegration, Integration};
use anyhow::Context;
use anyhow::Result;
use clap::Subcommand;
use std::path::PathBuf;

#[derive(Debug, Subcommand)]
pub enum SshSubcommand {
    /// Enable ssh integration
    Enable,
    /// Disable ssh integration
    Disable,
}

pub fn get_ssh_config_path() -> Result<PathBuf> {
    Ok(fig_directories::home_dir()
        .context("Could not get home directory")?
        .join(".ssh")
        .join("config"))
}

impl SshSubcommand {
    pub async fn execute(&self) -> Result<()> {
        let path = get_ssh_config_path()?;
        let integration = SshIntegration { path };
        match self {
            SshSubcommand::Enable => integration.install(None),
            SshSubcommand::Disable => integration.uninstall(),
        }
    }
}
