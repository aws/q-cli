use clap::Subcommand;

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum AutocompleteSubcommand {
    /// Update available autocomplete specs
    Update,
    /// Get information on latest specs release
    Latest,
    /// Get information on the current specs version
    Current,
}

impl AutocompleteSubcommand {
    pub async fn execute(&self) -> eyre::Result<()> {
        match self {
            AutocompleteSubcommand::Update => {
                fig_autocomplete::update_spec_store(false).await?;
                println!("Successfully updated spec store!");
                Ok(())
            },
            AutocompleteSubcommand::Latest => {
                let release = fig_api_client::autocomplete::AUTOCOMPLETE_REPO.latest_release().await?;
                println!("{}", release.tag_name);
                Ok(())
            },
            AutocompleteSubcommand::Current => {
                let tag_name =
                    fig_settings::state::get::<String>(fig_autocomplete::SETTINGS_SPEC_VERSION)?.unwrap_or_default();
                println!("{}", tag_name);
                Ok(())
            },
        }
    }
}
