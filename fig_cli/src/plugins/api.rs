use anyhow::{
    Context,
    Result,
};
use fig_settings::api_host;
use reqwest::Url;
use serde::{
    Deserialize,
    Serialize,
};

use super::manifest::GitHub;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginContext {
    install_directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ElementOrList<T> {
    Element(T),
    List(Vec<T>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginInstallData {
    pub source_files: Option<ElementOrList<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginData {
    pub name: String,
    pub github: Option<GitHub>,
    pub installation: Option<PluginInstallData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

pub async fn fetch_plugin(name: impl AsRef<str>) -> Result<PluginData> {
    let api_host = api_host();
    let name = name.as_ref();

    let url = Url::parse(&format!("{api_host}/plugins/name/{name}"))?;

    let body = reqwest::get(url).await?.error_for_status()?.text().await?;

    let data: PluginResponse = serde_json::from_str(&body)?;

    if data.success {
        Ok(data.plugin.context("Could not get plugin")?)
    } else {
        Err(anyhow::anyhow!("{}", data.message.unwrap()))
    }
}
