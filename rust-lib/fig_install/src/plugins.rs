use std::ffi::OsString;
use std::path::PathBuf;

use anyhow::{
    bail,
    Result,
};
use fig_api_client::plugins::plugin as fetch_plugin;
use fig_util::directories;
use tracing::{
    error,
    info,
};

use crate::git::update_git_repo_with_reference;
use crate::{
    dotfiles,
    git,
};

pub fn plugin_data_dir() -> Result<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            Ok(directories::home_dir().map(|dir| dir.join(".local").join("share").join("fig").join("plugins"))?)
        } else {
            Ok(directories::fig_data_dir()?.join("plugins"))
        }
    }
}

pub async fn fetch_installed_plugins(update: bool) -> Result<()> {
    let dotfiles_path = dotfiles::api::all_file_path()?;
    let dotfiles_file = std::fs::File::open(dotfiles_path)?;

    let dotfiles_data: dotfiles::api::DotfilesData = serde_json::from_reader(&dotfiles_file)?;

    let tasks = dotfiles_data.plugins.into_iter().map(|plugin| {
        tokio::spawn(async move {
            match fetch_plugin(plugin.name).await {
                Ok(plugin) => {
                    if let Ok(plugins_directory) = plugin_data_dir() {
                        let plugin_directory = plugins_directory.join(&plugin.name);

                        if !git::check_if_git_repo(&plugin_directory).await {
                            if let Some(github) = plugin.github {
                                match git::clone_git_repo_with_reference(github.git_url(), &plugin_directory, None)
                                    .await
                                {
                                    Ok(_) => info!("Cloned plugin {}", plugin.name),
                                    Err(err) => {
                                        error!("Error cloning plugin '{}': {err}", plugin.name);
                                        bail!("Error cloning plugin '{}': {err}", plugin.name);
                                    },
                                }
                            } else {
                                error!("No github url found for plugin '{}'", plugin.name);
                                bail!("No github url found for plugin '{}'", plugin.name);
                            }
                        } else if update {
                            match update_git_repo_with_reference(&plugin_directory, None).await {
                                Ok(_) => info!("Updated plugin '{}'", plugin.name),
                                Err(err) => {
                                    error!("Error updating plugin '{}': {err}", plugin.name);
                                    bail!("Error updating plugin '{}': {err}", plugin.name);
                                },
                            }
                        }

                        // Run zcompile
                        if fig_settings::settings::get_bool_or("plugins.zcompile", false) {
                            if let Some(installation) = plugin.installation {
                                if let Some(source_files) = installation.source_files {
                                    for file in source_files.iter() {
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
                Err(err) => {
                    error!("Error fetching plugin: {err}");
                    return Err(err.into());
                },
            }

            Ok(())
        })
    });

    let joined_errors: Vec<_> = futures::future::join_all(tasks)
        .await
        .into_iter()
        .filter_map(|res| match res {
            Ok(fetch_res) => match fetch_res {
                Ok(_) => None,
                Err(err) => Some(err),
            },
            // Ignore Join Errors
            Err(_join_error) => None,
        })
        .collect();

    if joined_errors.is_empty() {
        Ok(())
    } else {
        bail!(
            joined_errors
                .into_iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}
