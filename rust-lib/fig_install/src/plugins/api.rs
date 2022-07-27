use anyhow::Result;
use fig_request::Request;
use fig_util::Shell;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;

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
struct PluginDataResponse {
    plugin: PluginData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginData {
    pub name: String,
    pub display_name: Option<String>,
    pub icon: Option<String>,
    pub github: Option<GitHub>,
    pub installation: Option<PluginInstallData>,
    pub configuration: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstalledPlugin {
    name: String,
    shells: Option<ElementOrList<Shell>>,
    last_update: Option<u64>,
}

pub async fn fetch_plugin(name: impl std::fmt::Display) -> Result<PluginData> {
    let plugin_data_reponse: PluginDataResponse = Request::get(format!("/plugins/name/{name}"))
        .auth()
        .deser_json()
        .await?;
    Ok(plugin_data_reponse.plugin)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManyPlugins {
    plugins: Vec<serde_json::Map<String, serde_json::Value>>,
}

pub async fn all_plugins<F, I>(fields: F) -> Result<Vec<serde_json::Map<String, serde_json::Value>>>
where
    F: IntoIterator<Item = I>,
    I: Into<String>,
{
    let query = format!(
        "query {{ plugins {{ {} }} }}",
        fields
            .into_iter()
            .map(|field| field.into())
            .collect::<Vec<_>>()
            .join(" ")
    );

    let many_plugins: ManyPlugins = Request::post("/graphql")
        .body(json!({ "query": query }))
        .graphql()
        .await?;

    Ok(many_plugins.plugins)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UniquePlugin {
    plugin: serde_json::Map<String, serde_json::Value>,
}

pub async fn unique_plugin<N, F, I>(name: N, fields: F) -> Result<serde_json::Map<String, serde_json::Value>>
where
    N: std::fmt::Display,
    F: IntoIterator<Item = I>,
    I: Into<String>,
{
    let query = format!(
        "query {{ plugin ( where: {{ name: \"{name}\" }} ) {{ {} }} }}",
        fields
            .into_iter()
            .map(|field| field.into())
            .collect::<Vec<_>>()
            .join(" ")
    );

    let unique_plugin: UniquePlugin = Request::post("/graphql")
        .body(json!({ "query": query }))
        .graphql()
        .await?;

    Ok(unique_plugin.plugin)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstalledPlugins {
    plugins: Vec<serde_json::Map<String, serde_json::Value>>,
}

pub async fn installed_plugins<F, I>(_fields: F) -> Result<Vec<serde_json::Map<String, serde_json::Value>>>
where
    F: IntoIterator<Item = I>,
    I: Into<String>,
{
    let installed_plugins: InstalledPlugins = Request::get("/dotfiles/plugins").auth().deser_json().await?;
    Ok(installed_plugins.plugins)
}
