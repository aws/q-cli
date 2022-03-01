use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub async fn test() -> Result<()> {
    let plugin = fetch_plugin("oh-my-zsh").await?;
    println!("{:#?}", plugin);

    let cwd = std::env::current_dir()?;

    let a: Vec<_> = walkdir::WalkDir::new(&cwd)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|e| e.file_type().is_file())
        .map(|entry| entry.path().strip_prefix(&cwd).unwrap().to_owned())
        .collect();

    let a = json!(a);

    println!("{}", a);

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
    #[serde(rename = "use")]
    pub use_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginData {
    pub name: String,
    pub github: Option<String>,
    pub installation: Option<PluginInstallData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResponse {
    pub success: bool,
    pub plugin: Option<PluginData>,
    pub message: Option<String>,
}

pub async fn fetch_plugin(name: impl AsRef<str>) -> Result<PluginData> {
    let url = format!("https://api.fig.io/plugins/name/{}", name.as_ref());
    let body = reqwest::get(&url).await?.error_for_status()?.text().await?;
    println!("{:#?}", serde_json::from_str::<serde_json::Value>(&body));
    let data: PluginResponse = serde_json::from_str(&body)?;

    if data.success {
        Ok(data.plugin.unwrap())
    } else {
        Err(anyhow::anyhow!("{}", data.message.unwrap()))
    }
}
