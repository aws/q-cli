use anyhow::{Context, Result};
use tracing::{error, info};

use crate::{dotfiles, plugins::download::update_git_repo_with_reference};

use self::download::plugin_data_dir;

pub mod api;
pub mod download;
pub mod manifest;

pub async fn fetch_installed_plugins(update: bool) -> Result<()> {
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
                if let Some(plugins_directory) = plugin_data_dir() {
                    let plugin_directory = plugins_directory.join(&plugin.name);

                    let mut cloned = false;

                    if let Some(github) = plugin.github {
                        match download::clone_git_repo_with_reference(
                            github.git_url(),
                            &plugin_directory,
                            None,
                        )
                        .await
                        {
                            Ok(_) => {
                                info!("Cloned plugin {}", plugin.name);
                                cloned = true;
                            }
                            Err(err) => {
                                error!("Error cloning {}: {}", plugin.name, err)
                            }
                        }
                    } else {
                        error!("No github url found for plugin {}", plugin.name);
                    }

                    if !cloned && update {
                        match update_git_repo_with_reference(plugin_directory, None).await {
                            Ok(_) => info!("Updated plugin {}", plugin.name),
                            Err(err) => {
                                error!("Error updating plugin {}: {}", plugin.name, err);
                            }
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
