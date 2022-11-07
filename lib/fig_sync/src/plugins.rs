use std::ffi::OsString;

use fig_api_client::plugins::plugin as fetch_plugin;
use fig_util::directories;
use thiserror::Error;
use tracing::{
    error,
    info,
};

use crate::git::{
    update_git_repo_with_reference,
    GitError,
};
use crate::{
    dotfiles,
    git,
};

pub mod manifest;

pub type Result<T, E = PluginError> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct CollectedError(Vec<PluginError>);

impl std::fmt::Display for CollectedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for error in &self.0 {
            writeln!(f, "{error}")?
        }
        Ok(())
    }
}

impl std::error::Error for CollectedError {}

#[derive(Debug, Error)]
pub enum PluginError {
    #[error(transparent)]
    Request(#[from] fig_request::Error),
    #[error(transparent)]
    Git(#[from] GitError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Dir(#[from] fig_util::directories::DirectoryError),
    #[error("no git url found")]
    NoGitUrl,
    #[error(transparent)]
    Collected(#[from] CollectedError),
}

pub async fn fetch_installed_plugins(update: bool) -> Result<()> {
    let dotfiles_path = dotfiles::api::all_file_path()?;
    let dotfiles_file = std::fs::File::open(dotfiles_path)?;

    let dotfiles_data: dotfiles::api::DotfilesData = serde_json::from_reader(&dotfiles_file)?;

    let tasks = dotfiles_data.plugins.into_iter().map(|plugin| {
        tokio::spawn(async move {
            match fetch_plugin(plugin.name).await {
                Ok(plugin) => {
                    if let Ok(plugins_directory) = directories::plugins_dir() {
                        let plugin_directory = plugins_directory.join(&plugin.name);

                        if !git::check_if_git_repo(&plugin_directory).await {
                            if let Some(github) = plugin.github {
                                match git::clone_git_repo_with_reference(github.git_url(), &plugin_directory, None)
                                    .await
                                {
                                    Ok(_) => info!("Cloned plugin {}", plugin.name),
                                    Err(err) => {
                                        error!("Error cloning plugin '{}': {err}", plugin.name);
                                        return Err(err.into());
                                    },
                                }
                            } else {
                                error!("No github url found for plugin '{}'", plugin.name);
                                return Err(PluginError::NoGitUrl);
                            }
                        } else if update {
                            match update_git_repo_with_reference(&plugin_directory, None).await {
                                Ok(_) => info!("Updated plugin '{}'", plugin.name),
                                Err(err) => {
                                    error!("Error updating plugin '{}': {err}", plugin.name);
                                    return Err(err.into());
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
        Err(CollectedError(joined_errors.into_iter().collect::<Vec<_>>()).into())
    }
}
