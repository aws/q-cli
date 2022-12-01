use clap::{
    Args,
    Subcommand,
};
use fig_proto::local::UiElement;
use fig_util::desktop::{
    launch_fig_desktop,
    LaunchArgs,
};
use fig_util::directories::scripts_cache_dir;

#[derive(Debug, Args, PartialEq, Eq)]
pub struct ScriptsArgs {
    #[command(subcommand)]
    subcommand: Option<ScriptsSubcommands>,
}

#[derive(Debug, Subcommand, PartialEq, Eq)]
pub enum ScriptsSubcommands {
    Refresh,
}

impl ScriptsArgs {
    pub async fn execute(self) -> eyre::Result<()> {
        match self.subcommand {
            Some(ScriptsSubcommands::Refresh) => {
                tokio::fs::remove_dir_all(scripts_cache_dir()?).await?;
                fig_api_client::scripts::sync_scripts().await?;
                Ok(())
            },
            None => {
                launch_fig_desktop(LaunchArgs {
                    wait_for_socket: true,
                    open_dashboard: false,
                    immediate_update: true,
                    verbose: false,
                })
                .ok();

                if fig_ipc::local::open_ui_element(UiElement::MissionControl, Some("/scripts".into()))
                    .await
                    .is_err()
                {
                    eyre::bail!("Failed to open Scripts");
                }

                Ok(())
            },
        }
    }
}
