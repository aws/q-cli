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
    Named {
        name: String,
    },
    #[serde(rename_all = "camelCase")]
    Script {
        script: String,
    },
    Unknown(Option<String>),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FileType {
    #[default]
    Any,
    FileOnly,
    FolderOnly,
    Unknown(Option<String>),
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
        allow_raw_text_input: Option<bool>,
    },
    #[serde(rename_all = "camelCase")]
    Text {
        placeholder: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Checkbox {
        true_value_substitution: String,
        false_value_substitution: String,
    },
    #[serde(rename_all = "camelCase")]
    Path {
        #[serde(default)]
        file_type: FileType,
        #[serde(default)]
        extensions: Vec<String>,
    },
    Unknown(Option<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterCommandlineInterfaceType {
    Boolean { default: Option<bool> },
    String { default: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterCommandlineInterface {
    pub short: Option<String>,
    pub long: Option<String>,
    pub required: Option<bool>,
    pub require_equals: Option<bool>,
    pub r#type: Option<ParameterCommandlineInterfaceType>,
    pub raw: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub depends_on: Vec<String>,
    pub required: Option<bool>,
    #[serde(flatten)]
    pub parameter_type: ParameterType,
    pub cli: Option<ParameterCommandlineInterface>,
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
    Unknown(String),
}

impl Display for RuleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuleType::WorkingDirectory => f.write_str("Working directory"),
            RuleType::GitRemote => f.write_str("Git remote"),
            RuleType::ContentsOfDirectory => f.write_str("Contents of directory"),
            RuleType::GitRootDirectory => f.write_str("Git root directory"),
            RuleType::EnvironmentVariable => f.write_str("Environment"),
            RuleType::CurrentBranch => f.write_str("Current branch"),
            RuleType::Unknown(unknown) => write!(f, "Unknown: {unknown}"),
        }
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
    Unknown(String),
}

impl Display for Predicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Predicate::Contains => f.write_str("contain"),
            Predicate::Equals => f.write_str("equal"),
            Predicate::Matches => f.write_str("match with"),
            Predicate::StartsWith => f.write_str("start with"),
            Predicate::EndsWith => f.write_str("end with"),
            Predicate::Exists => f.write_str("exist"),
            Predicate::Unknown(unknown) => write!(f, "Unknown: {unknown}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub key: RuleType,
    pub specifier: Option<String>,
    pub predicate: Predicate,
    pub inverted: bool,
    pub value: Option<String>,
}

impl Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.key,
            match self.inverted {
                true => "must not",
                false => "must",
            },
            self.predicate,
        )?;
        match self.value {
            Some(ref value) if !value.is_empty() => {
                write!(f, " \"{value}\"")?;
            },
            _ => {},
        }
        Ok(())
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
    pub uuid: String,
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub template_version: i64,
    pub tags: Option<Vec<String>>,
    pub rules: Option<Vec<Vec<Rule>>>,
    pub namespace: String,
    pub parameters: Vec<Parameter>,
    pub tree: Vec<TreeElement>,
    pub is_owned_by_user: bool,
    #[serde(default)]
    pub runtime: Runtime,
    #[serde(default)]
    pub relevance: f64,
    #[serde(with = "time::serde::rfc3339::option", default)]
    pub last_invoked_at: Option<time::OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option", default)]
    pub last_invoked_at_by_user: Option<time::OffsetDateTime>,
    #[serde(default)]
    pub invocation_track_stderr: bool,
    #[serde(default)]
    pub invocation_track_stdout: bool,
    #[serde(default)]
    pub invocation_track_inputs: bool,
    #[serde(default)]
    pub invocation_disable_track: bool,
}

macro_rules! map_script {
    ($script:expr, $namespace:expr) => {{
        let script = $script;
        let namespace = $namespace;

        Script {
            uuid: script.uuid,
            name: script.name,
            namespace,
            display_name: script.fields.display_name,
            description: script.fields.description,
            template_version: script.fields.template_version.unwrap_or_default(),
            tags: script.fields.tags,
            rules: script.fields.ruleset.map(|r| {
                r.into_iter()
                    .map(|r| {
                        r.or.unwrap_or_default()
                            .into_iter()
                            .map(|rule| Rule {
                                key: match rule.key {
                                    ScriptRuleKey::ContentsOfDirectory => RuleType::ContentsOfDirectory,
                                    ScriptRuleKey::CurrentBranch => RuleType::CurrentBranch,
                                    ScriptRuleKey::EnvironmentVariable => RuleType::EnvironmentVariable,
                                    ScriptRuleKey::GitRemote => RuleType::GitRemote,
                                    ScriptRuleKey::GitRootDirectory => RuleType::GitRootDirectory,
                                    ScriptRuleKey::WorkingDirectory => RuleType::WorkingDirectory,
                                    ScriptRuleKey::Other(other) => RuleType::Unknown(other),
                                },
                                specifier: rule.specifier,
                                predicate: match rule.predicate {
                                    ScriptRulePredicate::Contains => Predicate::Contains,
                                    ScriptRulePredicate::EndsWith => Predicate::EndsWith,
                                    ScriptRulePredicate::Equals => Predicate::Equals,
                                    ScriptRulePredicate::Exists => Predicate::Exists,
                                    ScriptRulePredicate::Matches => Predicate::Matches,
                                    ScriptRulePredicate::StartsWith => Predicate::StartsWith,
                                    ScriptRulePredicate::Other(other) => Predicate::Unknown(other),
                                },
                                inverted: rule.inverted,
                                value: rule.value,
                            })
                            .collect()
                    })
                    .collect()
            }),
            parameters: script
                .fields
                .parameters
                .unwrap_or_default()
                .into_iter()
                .map(|parameter| Parameter {
                    name: parameter.name,
                    display_name: parameter.display_name,
                    description: parameter.description,
                    depends_on: vec![],
                    required: parameter.required,
                    parameter_type: match parameter.type_ {
                        ScriptParameterType::Checkbox => match parameter.checkbox {
                            Some(checkbox) => ParameterType::Checkbox {
                                true_value_substitution: checkbox
                                    .true_value_substitution
                                    .unwrap_or_else(|| "true".to_string()),
                                false_value_substitution: checkbox
                                    .false_value_substitution
                                    .unwrap_or_else(|| "false".to_string()),
                            },
                            None => ParameterType::Unknown(None),
                        },
                        ScriptParameterType::Path => match parameter.path {
                            Some(path) => ParameterType::Path {
                                file_type: match path.file_type {
                                    Some(ScriptFileType::Any) | None => FileType::Any,
                                    Some(ScriptFileType::FileOnly) => FileType::FileOnly,
                                    Some(ScriptFileType::FolderOnly) => FileType::FolderOnly,
                                    Some(ScriptFileType::Other(other)) => FileType::Unknown(Some(other)),
                                },
                                extensions: path.extensions.unwrap_or_default(),
                            },
                            None => ParameterType::Unknown(None),
                        },
                        ScriptParameterType::Selector => match parameter.selector {
                            Some(selector) => ParameterType::Selector {
                                placeholder: selector.placeholder,
                                suggestions: selector.suggestions,
                                generators: selector.generators.map(|generators| {
                                    generators
                                        .into_iter()
                                        .map(|generator| match generator.type_ {
                                            ScriptGeneratorType::Named => match generator.named {
                                                Some(generator) => Generator::Named { name: generator.name },
                                                None => Generator::Unknown(None),
                                            },
                                            ScriptGeneratorType::ShellScript => match generator.shell_script {
                                                Some(generator) => Generator::Script {
                                                    script: generator.script,
                                                },
                                                None => Generator::Unknown(None),
                                            },
                                            ScriptGeneratorType::Other(other) => Generator::Unknown(Some(other)),
                                        })
                                        .collect()
                                }),
                                allow_raw_text_input: selector.allow_raw_text_input,
                            },
                            None => ParameterType::Unknown(None),
                        },
                        ScriptParameterType::Text => match parameter.text {
                            Some(text) => ParameterType::Text {
                                placeholder: text.placeholder,
                            },
                            None => ParameterType::Unknown(None),
                        },
                        ScriptParameterType::Other(other) => ParameterType::Unknown(Some(other)),
                    },
                    cli: match parameter.commandline_interface {
                        Some(interface) => Some(ParameterCommandlineInterface {
                            short: interface.short,
                            long: interface.long,
                            required: interface.required,
                            require_equals: interface.require_equals,
                            raw: interface.raw,
                            r#type: match interface.type_ {
                                Some(ScriptCliBooleanType(t)) => Some(ParameterCommandlineInterfaceType::Boolean {
                                    default: t.boolean_default,
                                }),
                                Some(ScriptCliStringType(t)) => Some(ParameterCommandlineInterfaceType::String {
                                    default: t.string_default,
                                }),
                                None => None,
                            },
                        }),
                        None => None,
                    },
                })
                .collect(),
            tree: script
                .ast_tree
                .into_iter()
                .map(|tree| match tree.on {
                    ScriptFieldsAstTreeOn::ScriptAstParameter(param) => TreeElement::Token { name: param.name },
                    ScriptFieldsAstTreeOn::ScriptAstText(text) => TreeElement::String(text.text),
                })
                .collect(),
            runtime: match script.fields.runtime {
                Some(ScriptRuntime::Bash) => Runtime::Bash,
                Some(ScriptRuntime::Python) => Runtime::Python,
                Some(ScriptRuntime::Node) => Runtime::Node,
                Some(ScriptRuntime::Deno) => Runtime::Deno,
                Some(ScriptRuntime::Other(_)) | None => Runtime::default(),
            },
            relevance: script.relevance_score,
            is_owned_by_user: script.is_owned_by_current_user,
            last_invoked_at: script.last_invoked_at.map(|t| t.into()),
            last_invoked_at_by_user: script.last_invoked_at_by_user.map(|t| t.into()),
            invocation_disable_track: script
                .fields
                .invocation_collection
                .as_ref()
                .and_then(|s| s.disabled)
                .unwrap_or_default(),
            invocation_track_stdout: script
                .fields
                .invocation_collection
                .as_ref()
                .and_then(|s| s.stdout)
                .unwrap_or_default(),
            invocation_track_stderr: script
                .fields
                .invocation_collection
                .as_ref()
                .and_then(|s| s.stderr)
                .unwrap_or_default(),
            invocation_track_inputs: script
                .fields
                .invocation_collection
                .as_ref()
                .and_then(|s| s.inputs)
                .unwrap_or_default(),
        }
    }};
}

fn map_script(script: fig_graphql::script::ScriptFields, namespace: String) -> Script {
    use fig_graphql::script::ScriptFieldsFieldsParametersCommandlineInterfaceType::{
        ScriptParameterCommandlineInterfaceBoolean as ScriptCliBooleanType,
        ScriptParameterCommandlineInterfaceString as ScriptCliStringType,
    };
    use fig_graphql::script::{
        ScriptFieldsAstTreeOn,
        ScriptFileType,
        ScriptGeneratorType,
        ScriptParameterType,
        ScriptRuleKey,
        ScriptRulePredicate,
        ScriptRuntime,
    };

    map_script!(script, namespace)
}

fn map_scripts(script: fig_graphql::scripts::ScriptFields, namespace: String) -> Script {
    use fig_graphql::scripts::ScriptFieldsFieldsParametersCommandlineInterfaceType::{
        ScriptParameterCommandlineInterfaceBoolean as ScriptCliBooleanType,
        ScriptParameterCommandlineInterfaceString as ScriptCliStringType,
    };
    use fig_graphql::scripts::{
        ScriptFieldsAstTreeOn,
        ScriptFileType,
        ScriptGeneratorType,
        ScriptParameterType,
        ScriptRuleKey,
        ScriptRulePredicate,
        ScriptRuntime,
    };

    map_script!(script, namespace)
}

pub async fn script(
    namespace: impl Into<String>,
    name: impl Into<String>,
    _schema_version: i64,
) -> fig_request::Result<Option<Script>> {
    let namespace_str = namespace.into();

    let data = fig_graphql::script!(namespace: namespace_str.clone(), name: name).await?;

    let Some(namespace) = data.namespace else {
        return Ok(None);
    };

    let Some(script) = namespace.script else {
        return Ok(None);
    };

    Ok(Some(map_script(script, namespace_str)))
}

pub async fn scripts(_schema_version: i64) -> fig_request::Result<Vec<Script>> {
    let data = fig_graphql::scripts!().await?;
    let Some(current_user) = data.current_user else {
        return Ok(vec![]);
    };

    let mut scripts = vec![];

    if let Some(user_namespace) = current_user.namespace {
        for script in user_namespace.scripts {
            scripts.push((user_namespace.username.clone(), script));
        }
    }

    if let Some(team_memberships) = current_user.team_memberships {
        for team_membership in team_memberships {
            if let Some(namespace) = team_membership.team.namespace {
                for script in namespace.scripts {
                    scripts.push((namespace.username.clone(), script));
                }
            }
        }
    }

    Ok(scripts
        .into_iter()
        .map(|(namespace, script)| map_scripts(script, namespace))
        .collect())
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
