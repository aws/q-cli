//! Storage of data on the current downloaded plugins

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use tokio::fs::{read_to_string, write};

use crate::util::{glob, glob_dir, shell::Shell};

use super::{
    download::DownloadMetadata,
    manifest::{ShellInstall, StringOrList},
};

/// [ShellInstall] with the entries generated for the lock file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedShellInstall {
    /// Files after the glob pattern
    #[serde(rename = "use")]
    pub use_files: Option<Vec<PathBuf>>,
    /// List of templates to apply to the plugin
    pub apply: Option<Vec<String>>,
    /// Pre command to run before applying the plugin and other plugins that are sourced after this plugin
    pub pre: Option<Vec<String>>,
    /// Post command to run after applying the plugin and other plugins that are sourced after this plugin
    pub post: Option<Vec<String>>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockEntry {
    /// The unique name of the entry
    pub name: String,
    /// The version of the entry
    pub version: Option<String>,
    #[serde(flatten)]
    pub download_metadata: Option<DownloadMetadata>,
    #[serde(rename = "install")]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    pub shell_install: HashMap<Shell, LockedShellInstall>,
}

impl LockEntry {
    pub fn new(name: impl Into<String>) -> LockEntry {
        LockEntry {
            name: name.into(),
            version: None,
            download_metadata: None,
            shell_install: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LockData {
    #[serde(rename = "plugins")]
    entries: Vec<LockEntry>,
}

impl LockData {
    pub fn new() -> LockData {
        LockData {
            entries: Vec::new(),
        }
    }

    pub async fn load() -> Result<LockData> {
        let directory =
            super::download::plugin_data_dir().context("Failed to get source folder")?;
        let lock_path = directory.join("lock.toml");
        let raw = read_to_string(&lock_path).await?;
        Ok(toml::from_str(&raw)?)
    }

    pub async fn save(&self) -> Result<()> {
        let directory =
            super::download::plugin_data_dir().context("Failed to get source folder")?;
        let lock_path = directory.join("lock.toml");
        let data = toml::to_string(self)?;
        write(&lock_path, data).await?;
        Ok(())
    }

    pub fn add_entry(&mut self, entry: LockEntry) {
        self.entries.push(entry);
    }

    pub fn remove_entry(&mut self, name: impl AsRef<str>) {
        self.entries.retain(|entry| entry.name != name.as_ref());
    }

    pub fn get_entry(&self, name: impl AsRef<str>) -> Option<&LockEntry> {
        self.entries
            .iter()
            .find(|entry| entry.name == name.as_ref())
    }

    pub fn get_entry_mut(&mut self, name: impl AsRef<str>) -> Option<&mut LockEntry> {
        self.entries
            .iter_mut()
            .find(|entry| entry.name == name.as_ref())
    }

    pub fn get_entries(&self) -> &Vec<LockEntry> {
        &self.entries
    }

    pub fn get_entries_mut(&mut self) -> &mut Vec<LockEntry> {
        &mut self.entries
    }
}

const DEFAULT_ZSH_MATCH: &[&str] = &[
    "{{ name }}.plugin.zsh",
    "{{ name }}.zsh",
    "{{ name }}.sh",
    "{{ name }}.zsh-theme",
    "*.plugin.zsh",
    "*.zsh",
    "*.sh",
    "*.zsh-theme",
];

const DEFAULT_BASH_MATCH: &[&str] = &[
    "{{ name }}.plugin.bash",
    "{{ name }}.plugin.sh",
    "{{ name }}.bash",
    "{{ name }}.sh",
    "*.plugin.bash",
    "*.plugin.sh",
    "*.bash",
    "*.sh",
];

const DEFAULT_FISH_MATCH: &[&str] = &[
    "{{ name }}.plugin.fish",
    "{{ name }}.fish",
    "*.plugin.fish",
    "*.fish",
];

impl ShellInstall {
    pub fn use_files(&self, directory: impl AsRef<Path>, shell: &Shell) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        if let Some(use_files) = &self.use_files {
            let glob = glob(use_files)?;
            files.extend(glob_dir(&glob, directory)?);
        } else {
            let match_str = match shell {
                Shell::Zsh => DEFAULT_ZSH_MATCH,
                Shell::Bash => DEFAULT_BASH_MATCH,
                Shell::Fish => DEFAULT_FISH_MATCH,
            };

            let glob = glob(match_str)?;
            files.extend(glob_dir(&glob, directory)?);
        }

        println!("{:?}", files);

        Ok(files)
    }

    pub fn lock(&self, directory: impl AsRef<Path>, shell: &Shell) -> Result<LockedShellInstall> {
        let use_files = self.use_files(directory, shell)?;

        let use_files = match use_files.is_empty() {
            true => None,
            false => Some(use_files),
        };

        let pre = self.pre.as_ref().map_or(vec![], |post| match post {
            StringOrList::String(s) => vec![s.clone()],
            StringOrList::List(list) => list.clone(),
        });

        let pre = match pre.is_empty() {
            true => None,
            false => Some(pre),
        };

        let post = self.post.as_ref().map_or(vec![], |post| match post {
            StringOrList::String(s) => vec![s.clone()],
            StringOrList::List(list) => list.clone(),
        });

        let post = match post.is_empty() {
            true => None,
            false => Some(post),
        };

        Ok(LockedShellInstall {
            use_files,
            apply: self.apply.clone(),
            pre,
            post,
        })
    }
}
