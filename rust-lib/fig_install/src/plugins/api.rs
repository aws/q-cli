use anyhow::{
    Context,
    Result,
};
use fig_settings::api_host;
use fig_util::Shell;
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

impl<T> IntoIterator for ElementOrList<T> {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            ElementOrList::Element(e) => vec![e].into_iter(),
            ElementOrList::List(l) => l.into_iter(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginInstallData {
    pub source_files: Option<ElementOrList<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnInstallData {
    command: Option<ElementOrList<String>>,
    check: Option<OnInstallCheckData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnInstallCheckData {
    command_exists: Option<ElementOrList<String>>,
    file_exists: Option<ElementOrList<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnUninstallData {
    file: Option<ElementOrList<String>>,
    command: Option<ElementOrList<String>>,
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
    shells: Option<ElementOrList<Shell>>,
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
