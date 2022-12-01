use std::fmt::Display;

use fig_util::consts::FIG_SCRIPTS_SCHEMA_VERSION;
use serde::{
    Deserialize,
    Serialize,
};
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum Generator {
    #[serde(rename_all = "camelCase")]
    Named { name: String },
    #[serde(rename_all = "camelCase")]
    Script { script: String },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FileType {
    Any,
    FileOnly,
    FolderOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "typeData")]
#[serde(rename_all = "camelCase")]
pub enum ParameterType {
    #[serde(rename_all = "camelCase")]
    Selector {
        placeholder: Option<String>,
        suggestions: Option<Vec<String>>,
        generators: Option<Vec<Generator>>,
    },
    #[serde(rename_all = "camelCase")]
    Text { placeholder: Option<String> },
    #[serde(rename_all = "camelCase")]
    Checkbox {
        true_value_substitution: String,
        false_value_substitution: String,
    },
    #[serde(rename_all = "camelCase")]
    Path {
        file_type: FileType,
        extensions: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub depends_on: Vec<String>,
    #[serde(flatten)]
    pub parameter_type: ParameterType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleType {
    #[serde(rename = "Working-Directory")]
    WorkingDirectory,
    #[serde(rename = "Git-Remote")]
    GitRemote,
    #[serde(rename = "Contents-Of-Directory")]
    ContentsOfDirectory,
    #[serde(rename = "Git-Root-Directory")]
    GitRootDirectory,
    #[serde(rename = "Environment-Variable")]
    EnvironmentVariable,
    #[serde(rename = "Current-Branch")]
    CurrentBranch,
}

impl Display for RuleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            RuleType::WorkingDirectory => "Working directory",
            RuleType::GitRemote => "Git remote",
            RuleType::ContentsOfDirectory => "Contents of directory",
            RuleType::GitRootDirectory => "Git root directory",
            RuleType::EnvironmentVariable => "Environment",
            RuleType::CurrentBranch => "Current branch",
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Predicate {
    Contains,
    Equals,
    Matches,
    StartsWith,
    EndsWith,
    Exists,
}

impl Display for Predicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Predicate::Contains => "contain",
            Predicate::Equals => "equal",
            Predicate::Matches => "match with",
            Predicate::StartsWith => "start with",
            Predicate::EndsWith => "end with",
            Predicate::Exists => "exist",
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub key: RuleType,
    pub specifier: Option<String>,
    pub predicate: Predicate,
    pub inverted: bool,
    pub value: String,
}

impl Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}{}",
            self.key,
            match self.inverted {
                true => "must not",
                false => "must",
            },
            self.predicate,
            match self.value.is_empty() {
                true => "".to_owned(),
                false => format!(" \"{}\"", self.value),
            }
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum TreeElement {
    String(String),
    Token { name: String },
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Runtime {
    #[default]
    Bash,
    Python,
    Node,
    Deno,
}

impl Runtime {
    pub fn all() -> Vec<Self> {
        vec![Runtime::Bash, Runtime::Python, Runtime::Node, Runtime::Deno]
    }

    pub fn exe(&self) -> &str {
        match self {
            Runtime::Bash => "bash",
            Runtime::Python => "python3",
            Runtime::Node => "node",
            Runtime::Deno => "deno",
        }
    }

    pub fn brew_package(&self) -> &str {
        match self {
            Runtime::Bash => "bash",
            Runtime::Python => "python3",
            Runtime::Node => "node",
            Runtime::Deno => "deno",
        }
    }

    pub async fn version(&self) -> Option<String> {
        let regex = regex::Regex::new(match self {
            Runtime::Bash => r"version ([0-9.]+)",
            Runtime::Python => r"Python ([0-9.]+)",
            Runtime::Node => r"v([0-9.]+)",
            Runtime::Deno => r"deno ([0-9.]+)",
        })
        .ok()?;
        let output = Command::new(self.exe()).arg("--version").output().await.ok()?;
        let stdout = String::from_utf8(output.stdout).ok()?;
        Some(regex.captures(&stdout)?.get(1)?.as_str().to_owned())
    }
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Script {
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub template_version: u32,
    pub last_invoked_at: Option<String>,
    pub tags: Option<Vec<String>>,
    pub rules: Option<Vec<Vec<Rule>>>,
    pub namespace: String,
    pub parameters: Vec<Parameter>,
    pub template: String,
    pub tree: Vec<TreeElement>,
    pub is_owned_by_user: bool,
    #[serde(default)]
    pub runtime: Runtime,
    #[serde(default)]
    pub relevance: f64,
}

pub async fn script(namespace: &str, name: &str, schema_version: u32) -> fig_request::Result<Script> {
    fig_request::Request::get(format!("/workflows/{name}"))
        .query(&[
            ("namespace", namespace),
            ("schema-version", &schema_version.to_string()),
        ])
        .auth()
        .deser_json()
        .await
}

pub async fn scripts(schema_version: u32) -> fig_request::Result<Vec<Script>> {
    fig_request::Request::get("/workflows")
        .query(&[("schema-version", schema_version)])
        .auth()
        .deser_json()
        .await
}

/// Caches the scripts and returns them
pub async fn sync_scripts() -> fig_request::Result<Vec<Script>> {
    let scripts_cache_dir = fig_util::directories::scripts_cache_dir()?;
    tokio::fs::create_dir_all(&scripts_cache_dir).await?;

    let scripts = scripts(FIG_SCRIPTS_SCHEMA_VERSION).await?;

    // Delete old scripts so if one was removed from the server it will be removed locally
    if let Ok(mut read_dir) = tokio::fs::read_dir(&scripts_cache_dir).await {
        while let Some(entry) = read_dir.next_entry().await.ok().flatten() {
            tokio::fs::remove_file(entry.path()).await.ok();
        }
    }

    // Write new scripts
    for script in &scripts {
        tokio::fs::write(
            scripts_cache_dir.join(format!("{}.{}.json", script.namespace, script.name)),
            serde_json::to_string_pretty(&script)?.as_bytes(),
        )
        .await?;
    }

    Ok(scripts)
}

#[cfg(test)]
mod test {
    use super::*;

    #[ignore = "runtime may not be installed"]
    #[tokio::test]
    async fn test_version() {
        for runtime in Runtime::all() {
            println!("{:?} version: {}", runtime, runtime.version().await.unwrap());
        }
    }
}
