use std::fmt::Display;
use std::path::PathBuf;

use fig_util::directories::{
    home_dir,
    scripts_cache_dir,
};
use once_cell::sync::Lazy;
use regex::Regex;
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
        tree: Vec<TreeElement>,
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
        multi: Option<bool>,
    },
    #[serde(rename_all = "camelCase")]
    Text {
        placeholder: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Checkbox {
        false_toggle_display: Option<String>,
        false_value_substitution: String,
        true_toggle_display: Option<String>,
        true_value_substitution: String,
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

    pub fn exe(&self) -> PathBuf {
        match self {
            Runtime::Bash => "bash".into(),
            Runtime::Python => "python3".into(),
            Runtime::Node => "node".into(),
            Runtime::Deno => {
                // Try path
                if let Ok(deno) = which::which("deno") {
                    return deno;
                }

                // Try local install location
                let deno_install = match std::env::var_os("DENO_INSTALL") {
                    Some(deno_install) => Some(PathBuf::from(deno_install).join("bin").join("deno")),
                    None => home_dir().map(|home| home.join(".deno").join("bin").join("deno")).ok(),
                };

                if let Some(path) = deno_install {
                    if path.exists() {
                        return path;
                    }
                }

                // Fallback to just bin name
                "deno".into()
            },
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

    pub fn pacman_package(&self) -> &str {
        match self {
            Runtime::Bash => "bash",
            Runtime::Python => "python",
            Runtime::Node => "nodejs",
            Runtime::Deno => "deno",
        }
    }

    pub fn dnf_package(&self) -> Option<&str> {
        match self {
            Runtime::Bash => Some("bash"),
            Runtime::Python => Some("python3"),
            Runtime::Node => Some("nodejs"),
            Runtime::Deno => None,
        }
    }

    pub fn apt_package(&self) -> Option<&str> {
        match self {
            Runtime::Bash => Some("bash"),
            Runtime::Python => Some("python3"),
            Runtime::Node => None,
            Runtime::Deno => None,
        }
    }

    pub fn fallback_install_script(&self) -> Option<&str> {
        match self {
            Runtime::Bash => None,
            Runtime::Python => None,
            Runtime::Node => None,
            Runtime::Deno => Some("curl -fsSL https://deno.land/x/install/install.sh | sh"),
        }
    }

    /// Documentation for how to install the runtime
    pub fn install_docs(&self) -> Option<&str> {
        match self {
            Runtime::Bash => None,
            Runtime::Python => Some("https://wiki.python.org/moin/BeginnersGuide/Download/"),
            Runtime::Node => Some("https://nodejs.org/en/download/"),
            Runtime::Deno => Some("https://deno.land/manual/getting_started/installation/"),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum ScriptStep {
    CodeBlock {
        name: Option<String>,
        runtime: Runtime,
        tree: Vec<TreeElement>,
    },
    Inputs {
        name: Option<String>,
        parameters: Vec<Parameter>,
    },
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
    pub steps: Vec<ScriptStep>,
    pub namespace: String,
    pub is_owned_by_user: bool,
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
    #[serde(default)]
    pub should_cache: bool,
}

macro_rules! map_script {
    ($script:expr) => {{
        let script = $script;

        let map_ast = |ast: Option<Vec<AstNode>>| -> Vec<TreeElement> {
            ast.unwrap_or_default()
                .into_iter()
                .map(|tree| match tree.on {
                    AstNodeOn::ScriptAstParameter(param) => TreeElement::Token { name: param.name },
                    AstNodeOn::ScriptAstText(text) => TreeElement::String(text.text),
                })
                .collect()
        };

        let map_parameter = |parameter: ScriptParameter| Parameter {
            name: parameter.name,
            display_name: parameter.display_name,
            description: parameter.description,
            depends_on: vec![],
            required: parameter.required,
            parameter_type: match parameter.type_ {
                ScriptParameterType::Checkbox => match parameter.checkbox {
                    Some(checkbox) => ParameterType::Checkbox {
                        false_toggle_display: checkbox.false_toggle_display,
                        false_value_substitution: checkbox
                            .false_value_substitution
                            .unwrap_or_else(|| "false".to_string()),
                        true_toggle_display: checkbox.true_toggle_display,
                        true_value_substitution: checkbox.true_value_substitution.unwrap_or_else(|| "true".to_string()),
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
                                            tree: map_ast(generator.ast_tree),
                                        },
                                        None => Generator::Unknown(None),
                                    },
                                    ScriptGeneratorType::Other(other) => Generator::Unknown(Some(other)),
                                })
                                .collect()
                        }),
                        allow_raw_text_input: selector.allow_raw_text_input,
                        multi: selector.multi,
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
        };

        let map_runtime = |runtime: Option<ScriptRuntime>| match runtime {
            Some(ScriptRuntime::Bash) => Runtime::Bash,
            Some(ScriptRuntime::Python) => Runtime::Python,
            Some(ScriptRuntime::Node) => Runtime::Node,
            Some(ScriptRuntime::Deno) => Runtime::Deno,
            Some(ScriptRuntime::Other(_)) | None => Runtime::default(),
        };

        Script {
            uuid: script.uuid,
            name: script.name,
            namespace: script.namespace_name,
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
            steps: script
                .fields
                .steps
                .unwrap_or_default()
                .into_iter()
                .map(|step| match step.on {
                    StepOn::ScriptCodeBlockStep(code_block) => ScriptStep::CodeBlock {
                        name: code_block.name,
                        runtime: code_block
                            .code_block_data
                            .map(|data| map_runtime(data.runtime))
                            .unwrap_or_default(),
                        tree: map_ast(code_block.ast_tree),
                    },
                    StepOn::ScriptInputsStep(inputs) => ScriptStep::Inputs {
                        name: inputs.name,
                        parameters: inputs
                            .parameters
                            .unwrap_or_default()
                            .into_iter()
                            .map(map_parameter)
                            .collect(),
                    },
                })
                .collect(),
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
            should_cache: script.should_cache,
        }
    }};
}

fn map_script(script: fig_graphql::script::ScriptFields) -> Script {
    use fig_graphql::script::ParameterCommandlineInterfaceType::{
        ScriptParameterCommandlineInterfaceBoolean as ScriptCliBooleanType,
        ScriptParameterCommandlineInterfaceString as ScriptCliStringType,
    };
    use fig_graphql::script::{
        AstNode,
        AstNodeOn,
        Parameter as ScriptParameter,
        ScriptFileType,
        ScriptGeneratorType,
        ScriptParameterType,
        ScriptRuleKey,
        ScriptRulePredicate,
        ScriptRuntime,
        StepOn,
    };

    map_script!(script)
}

fn map_scripts(script: fig_graphql::scripts::ScriptFields) -> Script {
    use fig_graphql::scripts::ParameterCommandlineInterfaceType::{
        ScriptParameterCommandlineInterfaceBoolean as ScriptCliBooleanType,
        ScriptParameterCommandlineInterfaceString as ScriptCliStringType,
    };
    use fig_graphql::scripts::{
        AstNode,
        AstNodeOn,
        Parameter as ScriptParameter,
        ScriptFileType,
        ScriptGeneratorType,
        ScriptParameterType,
        ScriptRuleKey,
        ScriptRulePredicate,
        ScriptRuntime,
        StepOn,
    };

    map_script!(script)
}

/// GraphQL query to get a script by name and maybe namespace
pub async fn script(
    namespace: impl Into<Option<String>>,
    name: impl Into<String>,
) -> fig_request::Result<Option<Script>> {
    let namespace_str = namespace.into();

    let data = fig_graphql::script!(namespace: namespace_str.clone(), name: name).await?;

    let Some(script) = data.script else {
        return Ok(None);
    };

    Ok(Some(map_script(script)))
}

/// GraphQL query to get all scripts for the current user
pub async fn scripts() -> fig_request::Result<Vec<Script>> {
    let data = fig_graphql::scripts!().await?;
    let Some(current_user) = data.current_user else {
        return Ok(vec![]);
    };

    let mut scripts = vec![];

    if let Some(user_namespace) = current_user.namespace {
        for script in user_namespace.scripts {
            scripts.push(script);
        }
    }

    if let Some(team_memberships) = current_user.team_memberships {
        for team_membership in team_memberships {
            if let Some(namespace) = team_membership.team.namespace {
                for script in namespace.scripts {
                    scripts.push(script);
                }
            }
        }
    }

    Ok(scripts.into_iter().map(map_scripts).collect())
}

/// Determines whether or not the script cache should be used
///
/// Disable if `FIG_DISABLE_SCRIPT_CACHE` env var is set, if `script.cache` setting is set to false,
/// or if we're in WSL
pub fn use_cache() -> bool {
    if std::env::var_os("FIG_DISABLE_SCRIPT_CACHE").is_some() {
        return false;
    }

    if let Ok(Some(val)) = fig_settings::settings::get_bool("script.cache") {
        return val;
    }

    !fig_util::system_info::in_wsl()
}

/// Caches the scripts and returns them
pub async fn sync_scripts() -> fig_request::Result<Vec<Script>> {
    let scripts_cache_dir = scripts_cache_dir()?;
    tokio::fs::create_dir_all(&scripts_cache_dir).await?;

    let scripts = scripts().await?;

    // Delete old scripts so if one was removed from the server it will be removed locally
    if let Ok(mut read_dir) = tokio::fs::read_dir(&scripts_cache_dir).await {
        while let Some(entry) = read_dir.next_entry().await.ok().flatten() {
            tokio::fs::remove_file(entry.path()).await.ok();
        }
    }

    // Write new scripts
    for script in &scripts {
        if script.should_cache {
            tokio::fs::write(
                scripts_cache_dir.join(format!("{}.{}.json", script.namespace, script.name)),
                serde_json::to_string_pretty(&script)?.as_bytes(),
            )
            .await?;

            if script.is_owned_by_user {
                tokio::fs::write(
                    scripts_cache_dir.join(format!("{}.json", script.name)),
                    serde_json::to_string_pretty(&script)?.as_bytes(),
                )
                .await?;
            }
        }
    }

    Ok(scripts)
}

static FILE_NAME_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r".*\..*\.json").unwrap());

// Attempts to get cached scripts, then falls back to synced scripts
pub async fn get_cached_scripts() -> fig_request::Result<Vec<Script>> {
    if !use_cache() {
        return scripts().await;
    }

    let scripts_cache_dir = scripts_cache_dir()?;
    tokio::fs::create_dir_all(&scripts_cache_dir).await?;

    if scripts_cache_dir.read_dir()?.count() == 0 {
        sync_scripts().await?;
    }

    let mut scripts = vec![];
    for file in scripts_cache_dir.read_dir()?.flatten() {
        if let Some(name) = file.file_name().to_str() {
            if FILE_NAME_REGEX.is_match(name) {
                let script = serde_json::from_slice::<Script>(&tokio::fs::read(file.path()).await?);

                match script {
                    Ok(script) => scripts.push(script),
                    Err(err) => {
                        tracing::error!(%err, "Failed to parse script");

                        // If any file fails to parse, we should re-sync the scripts
                        return sync_scripts().await;
                    },
                }
            }
        }
    }

    Ok(scripts)
}

pub async fn sync_script(
    namespace: impl Into<Option<String>>,
    name: impl Into<String>,
) -> fig_request::Result<Option<Script>> {
    let namespace_str = namespace.into();

    let scripts_cache_dir = scripts_cache_dir()?;
    tokio::fs::create_dir_all(&scripts_cache_dir).await?;

    let script = script(namespace_str.clone(), name.into()).await?;

    match script {
        Some(script) => {
            if script.should_cache {
                tokio::fs::write(
                    scripts_cache_dir.join(format!(
                        "{}.{}.json",
                        namespace_str.clone().unwrap_or_default(),
                        script.name
                    )),
                    serde_json::to_string_pretty(&script)?.as_bytes(),
                )
                .await?;

                if script.is_owned_by_user {
                    tokio::fs::write(
                        scripts_cache_dir.join(format!("{}.json", script.name)),
                        serde_json::to_string_pretty(&script)?.as_bytes(),
                    )
                    .await?;
                }
            }

            Ok(Some(script))
        },
        None => Ok(None),
    }
}

pub async fn get_cached_script(
    namespace: impl Into<Option<String>>,
    name: impl Into<String>,
) -> fig_request::Result<Option<Script>> {
    if !use_cache() {
        return script(namespace, name).await;
    }

    let namespace = namespace.into();
    let name = name.into();
    let file_path = match &namespace {
        Some(namespace) => scripts_cache_dir()?.join(format!("{namespace}.{name}.json")),
        None => scripts_cache_dir()?.join(format!("{name}.json")),
    };

    if file_path.exists() {
        let script = serde_json::from_slice::<Script>(&tokio::fs::read(file_path).await?);

        match script {
            Ok(script) => Ok(Some(script)),
            Err(err) => {
                tracing::error!(%err, "Failed to parse script");
                // If any file fails to parse, we should re-sync the scripts
                sync_script(namespace, name).await
            },
        }
    } else {
        sync_script(namespace, name).await
    }
}

pub async fn delete_script(namespace: impl AsRef<str> + Display, name: impl Display) -> fig_request::Result<()> {
    let script_dir = fig_util::directories::scripts_cache_dir()?;

    tokio::fs::remove_file(script_dir.join(format!("{namespace}.{name}.json")))
        .await
        .ok();

    let individual_path = script_dir.join(format!("{name}.json"));
    if let Ok(script) = tokio::fs::read_to_string(&individual_path).await {
        match serde_json::from_str::<Script>(&script) {
            Ok(script) if script.namespace == namespace.as_ref() => {
                tokio::fs::remove_file(&individual_path).await.ok();
            },
            Ok(_) => {},
            Err(_) => {
                tokio::fs::remove_file(&individual_path).await.ok();
            },
        }
    }

    Ok(())
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
