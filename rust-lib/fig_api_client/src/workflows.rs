use std::fmt::Display;

use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum Generator {
    #[serde(rename_all = "camelCase")]
    Named { name: String },
    #[serde(rename_all = "camelCase")]
    Script { script: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "typeData")]
#[serde(rename_all = "camelCase")]
pub enum ParameterType {
    #[serde(rename_all = "camelCase")]
    Checkbox {
        true_value_substitution: String,
        false_value_substitution: String,
    },
    #[serde(rename_all = "camelCase")]
    Text { placeholder: Option<String> },
    #[serde(rename_all = "camelCase")]
    Selector {
        placeholder: Option<String>,
        suggestions: Option<Vec<String>>,
        generators: Option<Vec<Generator>>,
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
#[serde(rename_all = "camelCase")]
pub struct Workflow {
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
}

pub async fn workflows(schema_version: u32) -> fig_request::Result<Vec<Workflow>> {
    fig_request::Request::get("/workflows")
        .query(&[("schema-version", schema_version)])
        .auth()
        .deser_json()
        .await
}
