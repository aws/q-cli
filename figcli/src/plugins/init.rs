//! Responsible for generating the data used in `fig init` command

use crate::{
    plugins::lock::{LockData, LockEntry},
    util::shell::Shell,
};

use anyhow::Result;

impl LockData {
    pub fn plugin_source(&self, name: impl AsRef<str>, shell: &Shell) -> Result<String> {
        match self.get_entry(name.as_ref()) {
            Some(lock_entry) => Ok(lock_entry.plugin_source(shell)?),
            None => Err(anyhow::anyhow!("Plugin not found")),
        }
    }
}

impl LockEntry {
    pub fn plugin_source(&self, _shell: &Shell) -> Result<String> {
        let mut string = String::new();

        string.push_str("# Source plugin for ");
        string.push_str(&self.name);
        string.push('\n');

        // Get list of all files to be added to the plugin

        Ok(string)
    }
}
