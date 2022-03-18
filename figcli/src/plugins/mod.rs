use anyhow::{Context, Result};
use tracing::{error, info};

use crate::dotfiles;

pub mod api;
pub mod download;
pub mod manifest;

pub async fn fetch_installed_plugins() -> Result<()> {
    let dotfiles_path = dotfiles::api::all_file_path().context("Could not read all file")?;
    let dotfiles_file = std::fs::File::open(dotfiles_path)?;

    let dotfiles_data: dotfiles::api::DotfilesData = serde_json::from_reader(&dotfiles_file)?;

    let tasks = dotfiles_data
        .plugins
        .into_iter()
        .map(|plugin| tokio::spawn(async { api::fetch_plugin(plugin.name).await }))
        .collect::<Vec<_>>();

    for task in tasks {
        match task.await {
            Ok(Ok(plugin)) => {
                if let Some(plugins_directory) = fig_directories::fig_data_dir() {
                    let plugin_directory = plugins_directory.join("plugins").join(&plugin.name);

                    info!("Cloneing or updating {}", plugin.name);

                    if let Some(github) = plugin.github {
                        if let Err(err) = download::update_or_clone_git_repo(
                            github.git_url(),
                            &plugin_directory,
                            None,
                        )
                        .await
                        {
                            error!("Error updating or cloning {}: {}", plugin.name, err);
                        }
                    }
                }
            }
            Ok(Err(err)) => error!("Error fetching plugin: {}", err),
            Err(err) => error!("Error fetching plugin: {}", err),
        }
    }

    Ok(())
}
