use std::path::{Path, PathBuf};

use anyhow::Result;
use fig_auth::get_token;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{plugins::download::update_or_clone_git_repo, util::api::api_host};

use super::manifest::GitHub;

fn _walk_dir(dir: &Path) -> Result<Vec<PathBuf>> {
    let paths: Vec<_> = walkdir::WalkDir::new(&dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|e| e.file_type().is_file())
        .map(|entry| entry.path().strip_prefix(&dir).unwrap().to_owned())
        .collect();
    Ok(paths)
}

pub async fn test() -> Result<()> {
    fetch_installed_plugins().await?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginContext {
    install_directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInstallData {
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginData {
    pub name: String,
    pub github: Option<GitHub>,
    pub installation: Option<PluginInstallData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResponse {
    pub success: bool,
    pub plugin: Option<PluginData>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstalledPlugin {
    name: String,
    last_update: Option<u64>,
}

pub async fn fetch_installed_plugins() -> Result<()> {
    let token = get_token().await?;

    let url = Url::parse(&format!("{}/dotfiles/plugins", api_host()))?;

    let body = reqwest::Client::new()
        .get(url)
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let plugins: Vec<InstalledPlugin> = serde_json::from_str(&body)?;

    let tasks = plugins
        .into_iter()
        .map(|plugin| tokio::spawn(async { fetch_plugin(plugin.name).await }))
        .collect::<Vec<_>>();

    for task in tasks {
        match task.await {
            Ok(Ok(plugin)) => {
                if let Some(plugins_directory) = fig_directories::fig_data_dir() {
                    let plugin_directory = plugins_directory.join("plugins").join(&plugin.name);

                    info!("Cloneing or updating {}", plugin.name);

                    if let Some(github) = plugin.github {
                        if let Err(err) =
                            update_or_clone_git_repo(github.git_url(), &plugin_directory, None)
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

pub async fn fetch_plugin(name: impl AsRef<str>) -> Result<PluginData> {
    let api_host = api_host();
    let name = name.as_ref();

    let url = Url::parse(&format!("{api_host}/plugins/name/{name}"))?;

    let body = reqwest::get(url).await?.error_for_status()?.text().await?;

    let data: PluginResponse = serde_json::from_str(&body)?;

    if data.success {
        Ok(data.plugin.unwrap())
    } else {
        Err(anyhow::anyhow!("{}", data.message.unwrap()))
    }
}
