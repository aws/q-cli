use std::{collections::HashMap, fmt, path::PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use crate::util::{shell::Shell, terminal::Terminal};

/// GitHub repo
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
        let owner = parts
            .next()
            .ok_or_else(|| serde::de::Error::custom("missing owner"))?;
        let repo = parts
            .next()
            .ok_or_else(|| serde::de::Error::custom("missing repo"))?;
        Ok(GitHub {
            owner: owner.to_owned(),
            repo: repo.to_owned(),
        })
    }
}

impl fmt::Display for GitHub {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AuthorValue {
    /// Name of the author
    Name(String),
    /// Name of the author and other optional information
    Details {
        /// The name of the author
        name: String,
        /// The Twitter handle of the author
        twitter: Option<String>,
        /// The GitHub username of the author
        github: Option<String>,
    },
}

/// Category of a plugin
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PluginType {
    /// Shell plugin
    Shell,
    /// Theme plugin
    Theme,
    /// Special plugin
    Special,
}

/// Enum for full dependency info or just the name
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    /// Name of the dependency
    Name(String),
    /// Name of the dependency and other optional information
    Full {
        /// The name of the dependency
        name: String,
        /// If the dependency is optional
        optional: Option<bool>,
        /// The git repository of the dependency
        git: Option<Url>,
        /// The path to the dependency
        path: Option<PathBuf>,
    },
}

/// Metadata of a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Name of the plugin
    pub name: String,
    /// The type of plugin
    #[serde(rename = "type")]
    pub plugin_type: PluginType,
    /// Description of the plugin
    pub description: Option<String>,
    /// Current version of the plugin
    pub version: Option<String>,
    /// Link to a icon for the plugin
    pub icon: Option<Url>,
    /// Links to images for the plugin
    pub images: Option<Vec<Url>>,
    /// Link to the site for the plugin
    pub site: Option<Url>,
    /// Link to the documentation for the plugin
    pub docs: Option<Url>,
    /// GitHub identifier of the plugin (owner/repo)
    pub github: Option<GitHub>,
    /// Link to the repository for the plugin
    pub repository: Option<Url>,
    /// Link to the README for the plugin
    pub readme: Option<Url>,
    /// The twitter handle of the author
    pub twitter: Option<String>,
    /// Authors of the plugin
    pub authors: Option<Vec<AuthorValue>>,
    /// License of the plugin
    pub license: Option<Vec<String>>,
    /// Shells supported by the plugin
    pub shells: Option<Vec<Shell>>,
    /// Terminals supported by the plugin
    pub terminals: Option<Vec<Terminal>>,
    /// Tags of the plugin
    pub tags: Option<Vec<String>>,
    /// Dependencies of the plugin
    pub dependencies: Option<Vec<Dependency>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GitReference {
    Commit(String),
    Branch(String),
    Tag(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum GithubValue {
    /// The name of the github repository
    GithubRepo(GitHub),
    /// true if the github is same as the github in the metadata
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gist(String);

impl Gist {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ShellSource {
    Git {
        git: Url,
        #[serde(flatten)]
        reference: Option<GitReference>,
    },
    Github {
        github: GithubValue,
        #[serde(flatten)]
        reference: Option<GitReference>,
    },
    Local {
        path: PathBuf,
    },
    Gist {
        gist: Gist,
        checksum: Option<String>,
    },
    Remote {
        remote: Url,
        checksum: Option<String>,
    },
}

/// Rules on how to install a shell plugin
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ShellInstall {
    /// Files/Globs to source in the shell
    #[serde(rename = "use")]
    source: Option<Vec<String>>,
    /// List of templates to apply to the plugin
    apply: Option<Vec<String>>,
    /// Pre command to run before applying the plugin and other plugins that are sourced after this plugin
    pre: Option<String>,
    /// Post command to run after applying the plugin and other plugins that are sourced after this plugin
    post: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ShellInstallation {
    source: ShellSource,
    install: Option<ShellInstall>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SpecialInstallation {
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeInstallation {
    #[serde(flatten)]
    terminals: HashMap<Terminal, toml::Value>,
}

/// Installation for a plugin
///
/// This is used to both define the source and how to install any plugin
/// type supported by Fig plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Installation {
    /// Installation for a shell plugin
    shell: Option<ShellInstallation>,
    /// Installation for a special plugin
    special: Option<SpecialInstallation>,
    /// Installation for a theme plugin
    theme: Option<ThemeInstallation>,
    // /// Installation for an app plugin
    // App(AppInstallation),
}

/// Enviroment variable configuration types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "kebab-case")]
pub enum EnvironmentConfigType {
    Filepath {
        default: Option<String>,
    },
    Select {
        default: Option<String>,
        options: Vec<String>,
    },
    MultiSelect {
        default: Option<Vec<String>>,
        options: Vec<String>,
    },
    Bool {
        on: Option<String>,
        off: Option<String>,
        default: Option<bool>,
    },
}

/// Used to configure environment variables for a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfigValue {
    #[serde(flatten)]
    config_type: EnvironmentConfigType,
    description: Option<String>,
}

/// A map from environment variable name to the value configuration
pub type ConfigEnvironment = HashMap<String, EnvironmentConfigValue>;

/// Configuration for a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    interface: Option<String>,
    environment: Option<ConfigEnvironment>,
}

/// A Fig plugin
///
/// Specifies the metadata of a plugin, the installation instructions, and any configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    #[serde(rename = "plugin")]
    pub metadata: PluginMetadata,

    pub installation: Installation,

    pub config: Option<Config>,
}

#[derive(Error, Debug)]
enum PluginValidationError {
    #[error("Plugin name is missing or empty")]
    MissingName,
    #[error("Missing valid installation for plugin type {0:?}")]
    MissingInstallation(PluginType),
}

impl Plugin {
    /// Validate the plugin is a valid configuration
    pub fn validate(&self) -> Result<()> {
        // Ensure the plugin has a name
        if self.metadata.name.is_empty() {
            return Err(PluginValidationError::MissingName.into());
        }

        // Ensure the plugin type has a valid installation
        if self.metadata.plugin_type == PluginType::Shell && self.installation.shell.is_none() {
            return Err(PluginValidationError::MissingInstallation(PluginType::Shell).into());
        }

        if self.metadata.plugin_type == PluginType::Special && self.installation.special.is_none() {
            return Err(PluginValidationError::MissingInstallation(PluginType::Special).into());
        }

        if self.metadata.plugin_type == PluginType::Theme && self.installation.theme.is_none() {
            return Err(PluginValidationError::MissingInstallation(PluginType::Theme).into());
        }

        Ok(())
    }

    pub fn normalize(&mut self) {
        if let Some(github) = &self.metadata.github {
            if self.metadata.repository == None {
                self.metadata.repository = Some(github.repository_url());
            }

            if self.metadata.readme == None {
                self.metadata.readme = Some(github.readme_url());
            }

            if let Some(ref mut installation) = self.installation.shell {
                if let ShellSource::Github {
                    github: ref mut github_val,
                    ..
                } = installation.source
                {
                    if *github_val == GithubValue::Bool(true) {
                        *github_val = GithubValue::GithubRepo(github.clone());
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_deseralize() {
        let plugin_toml = r#"
        [plugin]
        name = "pure"
        type = "shell"
        description = "Pretty, minimal and fast ZSH prompt"
        github = "sindresorhus/pure"
        authors = [ { name = "Sindre Sorhus", twitter = "sindresorhus", github = "sindresorhus" } ]
        license = ["MIT"]
        shells = ["zsh"]
        tags = ["zsh", "theme"]
        
        [installation.shell]
        source = { github = true }
        install = { use = ["async.zsh", "pure.zsh"] }
        "#;

        let plugin: Plugin = toml::from_str(plugin_toml).unwrap();

        assert_eq!(plugin.metadata.name, "pure");
        assert_eq!(plugin.metadata.plugin_type, PluginType::Shell);
        assert_eq!(
            plugin.metadata.description,
            Some("Pretty, minimal and fast ZSH prompt".to_string())
        );
        assert_eq!(
            plugin.metadata.github,
            Some(GitHub::new("sindresorhus", "pure"))
        );
        assert_eq!(
            plugin.metadata.authors,
            Some(vec![AuthorValue::Details {
                name: "Sindre Sorhus".to_string(),
                twitter: Some("sindresorhus".to_string()),
                github: Some("sindresorhus".to_string()),
            }])
        );
        assert_eq!(plugin.metadata.license, Some(vec!["MIT".to_string()]));
        assert_eq!(
            plugin.metadata.tags,
            Some(vec!["zsh".to_string(), "theme".to_string()])
        );

        assert_eq!(
            plugin.installation.shell.as_ref().unwrap().source,
            ShellSource::Github {
                github: GithubValue::Bool(true),
                reference: None,
            }
        );
        assert_eq!(
            plugin.installation.shell.as_ref().unwrap().install,
            Some(ShellInstall {
                source: Some(vec!["async.zsh".to_string(), "pure.zsh".to_string()]),
                ..Default::default()
            })
        );
    }

    #[test]
    fn test_shellsource() {
        let plugin_toml = r#"
        git = "http://git.com/foo/bar"
        commit = "abc123"
        "#;

        let source: ShellSource = toml::from_str(plugin_toml).unwrap();

        assert_eq!(
            source,
            ShellSource::Git {
                git: Url::parse("http://git.com/foo/bar").unwrap(),
                reference: Some(GitReference::Commit(String::from("abc123"))),
            }
        );

        let plugin_toml = r#"
        github = "sindresorhus/pure"
        branch = "master"
        "#;

        let source: ShellSource = toml::from_str(plugin_toml).unwrap();

        assert_eq!(
            source,
            ShellSource::Github {
                github: GithubValue::GithubRepo(GitHub::new("sindresorhus", "pure")),
                reference: Some(GitReference::Branch(String::from("master"))),
            }
        );

        let plugin_toml = r#"
        github = true
        tag = "1.0"
        "#;

        let source: ShellSource = toml::from_str(plugin_toml).unwrap();

        assert_eq!(
            source,
            ShellSource::Github {
                github: GithubValue::Bool(true),
                reference: Some(GitReference::Tag(String::from("1.0"))),
            }
        );

        let plugin_toml = r#"
        path = "~/.zsh/plugins/pure"
        "#;

        let source: ShellSource = toml::from_str(plugin_toml).unwrap();

        assert_eq!(
            source,
            ShellSource::Local {
                path: PathBuf::from("~/.zsh/plugins/pure"),
            }
        );

        let plugin_toml = r#"
        gist = "12345"
        checksum = "abc123"
        "#;

        let source: ShellSource = toml::from_str(plugin_toml).unwrap();

        assert_eq!(
            source,
            ShellSource::Gist {
                gist: Gist::new("12345"),
                checksum: Some(String::from("abc123")),
            }
        );

        let plugin_toml = r#"
        remote = "https://example.com/foo/bar.tar.gz"
        "#;

        let source: ShellSource = toml::from_str(plugin_toml).unwrap();

        assert_eq!(
            source,
            ShellSource::Remote {
                remote: Url::parse("https://example.com/foo/bar.tar.gz").unwrap(),
                checksum: None,
            }
        );

        let plugin_toml = "";
        let source: Result<ShellSource, _> = toml::from_str(plugin_toml);
        assert!(source.is_err());
    }

    #[test]
    fn test_shellinstall() {
        let plugin_toml = r#"
        use = ["async.zsh", "pure.zsh"]
        apply = ["PATH"]
        pre = "echo 'hello'"
        post = "echo 'goodbye'"
        "#;

        let install: ShellInstall = toml::from_str(plugin_toml).unwrap();

        assert_eq!(
            install,
            ShellInstall {
                source: Some(vec!["async.zsh".to_string(), "pure.zsh".to_string()]),
                apply: Some(vec!["PATH".to_string()]),
                pre: Some(String::from("echo 'hello'")),
                post: Some(String::from("echo 'goodbye'")),
            }
        );

        let plugin_toml = "";
        let install: ShellInstall = toml::from_str(plugin_toml).unwrap();

        assert_eq!(
            install,
            ShellInstall {
                source: None,
                apply: None,
                pre: None,
                post: None,
            }
        );
    }
}
