use anyhow::Result;
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum PluginsSubcommand {
    List,
    Test,
}

impl PluginsSubcommand {
    pub async fn execute(&self) -> Result<()> {
        match self {
            PluginsSubcommand::List => {}
            PluginsSubcommand::Test => {
                crate::plugins::api::test().await?;
            }
        }

        Ok(())
    }
}
