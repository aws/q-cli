use fig_request::{
    Request,
    Result,
};
use fig_util::Shell;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use url::Url;

use crate::util::ElementOrList;

/// A Github repo with the form `"owner/repo"`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHub {
    pub owner: String,
    pub repo: String,
}

impl GitHub {
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
            repo: repo.into(),
        }
    }
}

impl Serialize for GitHub {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{}/{}", self.owner, self.repo))
    }
}

impl<'de> Deserialize<'de> for GitHub {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let mut parts = s.split('/');
        let owner = parts.next().ok_or_else(|| serde::de::Error::custom("missing owner"))?;
        let repo = parts.next().ok_or_else(|| serde::de::Error::custom("missing repo"))?;
        Ok(GitHub {
            owner: owner.to_owned(),
            repo: repo.to_owned(),
        })
    }
}

impl std::fmt::Display for GitHub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.owner, self.repo)
    }
}

impl GitHub {
    pub fn readme_url(&self) -> Url {
        Url::parse(&format!(
            "https://raw.githubusercontent.com/{}/{}/HEAD/README.md",
            self.owner, self.repo
        ))
        .unwrap()
    }

    pub fn repository_url(&self) -> Url {
        Url::parse(&format!("https://github.com/{}/{}", self.owner, self.repo)).unwrap()
    }

    pub fn git_url(&self) -> Url {
        Url::parse(&format!("https://github.com/{}/{}.git", self.owner, self.repo)).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gist(String);

impl Gist {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn id(&self) -> &str {
        &self.0
    }

    pub fn raw_url(&self) -> Url {
        Url::parse(&format!("https://gist.githubusercontent.com/raw/{}", self.0)).unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginContext {
    install_directory: String,
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
pub struct PluginDataResponse {
    pub plugin: PluginData,
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
pub struct InstalledPlugin {
    pub name: String,
    pub shells: Option<ElementOrList<Shell>>,
    pub last_update: Option<u64>,
}

pub async fn plugin(name: impl std::fmt::Display) -> Result<PluginData> {
    let plugin_data_response: PluginDataResponse = Request::get(format!("/plugins/name/{name}"))
        .auth()
        .deser_json()
        .await?;
    Ok(plugin_data_response.plugin)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManyPlugins {
    plugins: Vec<serde_json::Map<String, serde_json::Value>>,
}

pub async fn all_plugins<F, I>(fields: F) -> Result<Vec<serde_json::Map<String, serde_json::Value>>, fig_request::Error>
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

pub async fn unique_plugin<N, F, I>(
    name: N,
    fields: F,
) -> Result<serde_json::Map<String, serde_json::Value>, fig_request::Error>
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

pub async fn installed_plugins<F, I>(
    _fields: F,
) -> Result<Vec<serde_json::Map<String, serde_json::Value>>, fig_request::Error>
where
    F: IntoIterator<Item = I>,
    I: Into<String>,
{
    let installed_plugins: InstalledPlugins = Request::get("/dotfiles/plugins").auth().deser_json().await?;
    Ok(installed_plugins.plugins)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_github() -> GitHub {
        GitHub::new("withfig", "autocomplete")
    }

    fn mock_gist() -> Gist {
        Gist::new("2203becba6e69ec1b01ae213015077a1")
    }

    #[test]
    fn github_urls() {
        let gh = mock_github();
        assert_eq!(
            gh.readme_url().as_str(),
            "https://raw.githubusercontent.com/withfig/autocomplete/HEAD/README.md"
        );
        assert_eq!(gh.repository_url().as_str(), "https://github.com/withfig/autocomplete");
        assert_eq!(gh.git_url().as_str(), "https://github.com/withfig/autocomplete.git");
    }

    #[test]
    fn gist_urls() {
        let gist = mock_gist();
        assert_eq!(
            gist.raw_url().as_str(),
            "https://gist.githubusercontent.com/raw/2203becba6e69ec1b01ae213015077a1"
        );
    }
}
