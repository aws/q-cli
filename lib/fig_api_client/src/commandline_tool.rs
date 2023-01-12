use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;

use fig_graphql::commandline_tool::{
    CommandFields,
    CommandFieldsOn,
    CommandFieldsOnScriptCommand,
    CommandFieldsOnScriptCommandScript,
};
use fig_request::Result;
use fig_util::directories::cache_dir;
use serde::{
    Deserialize,
    Serialize,
};
use tokio::task::JoinHandle;
use tracing::error;

pub async fn commandline_tool(namespace: impl Into<String>, name: impl Into<String>) -> Result<Option<CommandTree>> {
    let data = fig_graphql::commandline_tool! {
        namespace: namespace,
        name: name,
    }
    .await?;

    let cli_tool = match data.namespace {
        Some(namespace) => match namespace.commandline_tool {
            Some(cli_tool) => cli_tool,
            None => return Ok(None),
        },
        None => return Ok(None),
    };

    let mut commands = cli_tool
        .flattened_commands
        .into_iter()
        .map(|command| (command.uuid.clone(), command))
        .collect::<BTreeMap<String, CommandFields>>();

    let Some(root_command) = commands.remove(&cli_tool.root.uuid) else {
        return Ok(None);
    };

    Ok(Some(create_command_tree(root_command, &mut commands)))
}

pub async fn fetch_and_cache_command_line_tool(namespace: &str, name: &str) -> Result<Option<CommandTree>> {
    let command_tree = commandline_tool(namespace, name).await?;

    if let Some(command_tree) = &command_tree {
        command_tree.save_cache(namespace, name)?;
    } else {
        CommandTree::remove_cache(namespace, name)?;
    }

    Ok(command_tree)
}

pub async fn fetch_and_cache_all_command_line_tools() -> Result<()> {
    let response = fig_graphql::list_commandline_tools!().await?;

    // Delete all cached command line tools
    let cli_cache_dir = cache_dir()?.join("commandline_tool");
    match tokio::fs::read_dir(&cli_cache_dir).await {
        Ok(mut read_dir) => {
            while let Ok(Some(entry)) = read_dir.next_entry().await {
                if let Err(err) = tokio::fs::remove_file(entry.path()).await {
                    error!(%err, path =? entry.path(), "Failed to delete cache file");
                }
            }
        },
        Err(err) => error!(%err, ?cli_cache_dir, "Failed to read cache directory"),
    }

    let mut commandline_tools: Vec<String> = vec![];

    if let Some(current_user) = response.current_user {
        if let Some(namespace) = current_user.namespace {
            for cli in namespace.commandline_tools {
                commandline_tools.push(cli.root.name.clone());
                if let Err(err) = fetch_and_cache_command_line_tool(&namespace.username, &cli.root.name).await {
                    error!(%err, namespace =% namespace.username, cli_name =% cli.root.name, "Failed to fetch command line tool");
                }
            }
        }

        if let Some(team_memberships) = current_user.team_memberships {
            for team_membership in team_memberships {
                if let Some(namespace) = team_membership.team.namespace {
                    for cli in namespace.commandline_tools {
                        commandline_tools.push(cli.root.name.clone());
                        if let Err(err) = fetch_and_cache_command_line_tool(&namespace.username, &cli.root.name).await {
                            error!(%err, namespace =% namespace.username, cli_name =% cli.root.name, "Failed to fetch command line tool");
                        }
                    }
                }
            }
        }
    }

    // Read the file and diff to see if there are any CLIs that are no longer
    let cache_index_path = cache_dir()?.join("cli_cache_index.json");
    if let Ok(read_index) = tokio::fs::read_to_string(&cache_index_path).await {
        let old_commandline_tools: Vec<String> = serde_json::from_str(&read_index).unwrap_or_default();
        for old_cli in old_commandline_tools {
            if !commandline_tools.contains(&old_cli.to_string()) {
                if let Ok(script_path) = CommandTree::script_path(&old_cli) {
                    if let Err(err) = tokio::fs::remove_file(&script_path).await {
                        error!(%err, ?script_path, "Failed to remove old script executable");
                    }
                }
            }
        }
    }

    match serde_json::to_string(&commandline_tools) {
        Ok(contents) => {
            if let Err(err) = tokio::fs::write(&cache_index_path, contents).await {
                error!(%err, ?cache_index_path, "Failed to write cache index");
            }
        },
        Err(err) => error!(%err, ?cache_index_path, "Failed to serialize cache index"),
    }

    Ok(())
}

pub async fn cached_commandline_tool(
    namespace: impl Into<String>,
    name: impl Into<String>,
) -> (
    Result<Option<CommandTree>>,
    Option<JoinHandle<Result<Option<CommandTree>>>>,
) {
    let namespace = namespace.into();
    let name = name.into();

    let cache_path = CommandTree::cache_path(&namespace, &name).unwrap();

    'failed: {
        if cache_path.exists() {
            let Ok(cache_file) = std::fs::File::open(cache_path) else {
                break 'failed;
            };
            let cache_reader = std::io::BufReader::new(cache_file);
            let Ok(command_tree) = serde_json::from_reader::<_, CommandTree>(cache_reader) else {
                break 'failed;
            };

            let refresh_handle =
                tokio::spawn(async move { fetch_and_cache_command_line_tool(&namespace, &name).await });

            return (Ok(Some(command_tree)), Some(refresh_handle));
        }
    }

    // Fall back to fetching the command tree from the server
    let command_tree = fetch_and_cache_command_line_tool(&namespace, &name).await;
    (command_tree, None)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum CommandTree {
    #[serde(rename_all = "camelCase")]
    NestedCommand {
        uuid: String,
        name: String,
        description: Option<String>,
        subcommands: BTreeMap<String, CommandTree>,
    },
    #[serde(rename_all = "camelCase")]
    ScriptCommand {
        uuid: String,
        name: String,
        description: Option<String>,
        script_namespace: String,
        script_name: String,
    },
}

impl CommandTree {
    pub fn name(&self) -> &str {
        match self {
            CommandTree::NestedCommand { name, .. } => name,
            CommandTree::ScriptCommand { name, .. } => name,
        }
    }

    pub fn script_path(name: &str) -> Result<PathBuf> {
        Ok(fig_util::directories::home_dir()?.join(".local").join("bin").join(name))
    }

    pub fn write_script(&self, namespace: &str, name: &str) -> Result<()> {
        let script = format!("#!/usr/bin/env bash\nfig cli @{namespace}/{name} \"$@\"\n",);
        let script_path = Self::script_path(name)?;

        if let Some(parent) = script_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut open_options = std::fs::File::options();
        open_options.create(true).write(true).truncate(true);

        #[cfg(unix)]
        {
            use std::os::unix::prelude::OpenOptionsExt;
            open_options.mode(0o755);
        }

        let mut script_file = open_options.open(script_path)?;
        script_file.write_all(script.as_bytes())?;

        Ok(())
    }

    pub fn cache_path(
        namespace: &str,
        name: &str,
    ) -> Result<std::path::PathBuf, fig_util::directories::DirectoryError> {
        Ok(cache_dir()?
            .join("commandline_tool")
            .join(format!("{namespace}.{name}.json")))
    }

    pub fn save_cache(&self, namespace: &str, name: &str) -> Result<()> {
        let cache_path = Self::cache_path(namespace, name)?;

        if let Err(err) = self.write_script(namespace, name) {
            error!(%err, "Failed to write script");
        }

        if let Some(parent) = cache_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let cache_file = std::fs::File::create(cache_path)?;
        let cache_writer = std::io::BufWriter::new(cache_file);

        serde_json::to_writer_pretty(cache_writer, self)?;

        Ok(())
    }

    pub fn remove_cache(namespace: &str, name: &str) -> Result<()> {
        let cache_path = Self::cache_path(namespace, name)?;
        if cache_path.exists() {
            if let Err(err) = std::fs::remove_file(cache_path) {
                error!(%err, "Failed to remove cache file");
            }
        }

        let bin_path = Self::script_path(name)?;
        if bin_path.exists() {
            if let Err(err) = std::fs::remove_file(bin_path) {
                error!(%err, "Failed to remove script file");
            }
        }

        Ok(())
    }

    pub fn load_cache(namespace: &str, name: &str) -> Result<Self> {
        let cache_path = Self::cache_path(namespace, name)?;

        let cache_file = std::fs::File::open(cache_path)?;
        let cache_reader = std::io::BufReader::new(cache_file);

        Ok(serde_json::from_reader(cache_reader)?)
    }
}

fn create_command_tree(root: CommandFields, map: &mut BTreeMap<String, CommandFields>) -> CommandTree {
    match root.on {
        CommandFieldsOn::NestedCommand(nested) => {
            let mut subcommands = BTreeMap::new();

            for subcommand in &nested.subcommands {
                let subcommand_tree = create_command_tree(
                    map.remove(&subcommand.uuid).expect("Subcommand for uuid not found"),
                    map,
                );
                subcommands.insert(subcommand_tree.name().to_owned(), subcommand_tree);
            }

            CommandTree::NestedCommand {
                uuid: root.uuid,
                name: root.name,
                description: root.description,
                subcommands,
            }
        },
        CommandFieldsOn::ScriptCommand(CommandFieldsOnScriptCommand {
            script:
                CommandFieldsOnScriptCommandScript {
                    name: script_name,
                    namespace,
                },
        }) => CommandTree::ScriptCommand {
            uuid: root.uuid,
            name: root.name,
            description: root.description,
            script_name,
            script_namespace: namespace.expect("No namespace found for script").username,
        },
    }
}
