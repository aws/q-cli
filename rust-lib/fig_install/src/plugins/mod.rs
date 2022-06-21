use std::ffi::OsString;
use std::path::PathBuf;

use anyhow::{
    Context,
    Result,
};
use tracing::{
    error,
    info,
};

use crate::download::update_git_repo_with_reference;
use crate::{
    dotfiles,
    download,
};

pub mod api;
pub mod manifest;

pub fn plugin_data_dir() -> Option<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            fig_directories::home_dir().map(|dir| dir.join(".local").join("share").join("fig").join("plugins"))
        } else {
            fig_directories::fig_data_dir().map(|dir| dir.join("plugins"))
        }
    }
}

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
        tokio::spawn(async move {
            match task.await {
                Ok(Ok(plugin)) => {
                    if let Some(plugins_directory) = plugin_data_dir() {
                        let plugin_directory = plugins_directory.join(&plugin.name);

                        let mut cloned = false;

                        if let Some(github) = plugin.github {
                            match download::clone_git_repo_with_reference(github.git_url(), &plugin_directory, None)
                                .await
                            {
                                Ok(_) => {
                                    info!("Cloned plugin {}", plugin.name);
                                    cloned = true;
                                },
                                Err(err) => {
                                    error!("Error cloning {}: {}", plugin.name, err)
                                },
                            }
                        } else {
                            error!("No github url found for plugin {}", plugin.name);
                        }

                        if !cloned && update {
                            match update_git_repo_with_reference(&plugin_directory, None).await {
                                Ok(_) => info!("Updated plugin {}", plugin.name),
                                Err(err) => {
                                    error!("Error updating plugin {}: {}", plugin.name, err);
                                },
                            }
                        }

                        // Run zcompile
                        if fig_settings::settings::get_bool_or("plugins.zcompile", false) {
                            if let Some(installation) = plugin.installation {
                                if let Some(source_files) = installation.source_files {
                                    let source_files = match source_files {
                                        api::ElementOrList::Element(element) => vec![element],
                                        api::ElementOrList::List(v) => v,
                                    };

                                    for file in source_files {
                                        let mut argument = OsString::from("zcompile ");
                                        argument.push(plugin_directory.join(file));

                                        tokio::process::Command::new("zsh")
                                            .arg("-c")
                                            .arg(argument)
                                            .output()
                                            .await
                                            .ok();
                                    }
                                }
                            }
                        }
                    }
                },
                Ok(Err(err)) => error!("Error fetching plugin: {}", err),
                Err(err) => error!("Error fetching plugin: {}", err),
            }
        });

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    Ok(())
}
