use std::collections::BTreeMap;

use clap::{
    Args,
    Command,
};
use eyre::{
    bail,
    Result,
};
use fig_graphql::commandline_tool::{
    CommandFields,
    CommandFieldsOn,
    CommandFieldsOnScriptCommand,
    CommandFieldsOnScriptCommandScript,
};

use super::run;

#[derive(Debug, Args, PartialEq, Eq)]
pub struct CliArgs {
    #[arg(allow_hyphen_values = true)]
    args: Option<Vec<String>>,
}

impl CliArgs {
    pub async fn execute(self) -> Result<()> {
        match self.args {
            Some(args) => {
                let (namespace, name) = match args[0].strip_prefix('@') {
                    Some(name) => match name.split('/').collect::<Vec<&str>>()[..] {
                        [namespace, name] => (Some(namespace), name),
                        _ => bail!("Malformed CLI specifier, expects @<namespace>/<cli> <args>",),
                    },
                    None => bail!("Malformed CLI specifier, expects @<namespace>/<cli> <args>"),
                };

                let data = fig_graphql::commandline_tool! {
                    namespace: namespace.expect("No namespace provided"),
                    name: name,
                }
                .await?;

                let cli_tool = data
                    .namespace
                    .expect("No namespace found")
                    .commandline_tool
                    .expect("No CLI tool found");

                let mut commands = cli_tool
                    .flattened_commands
                    .into_iter()
                    .map(|command| (command.uuid.clone(), command))
                    .collect::<BTreeMap<String, CommandFields>>();

                let root_command = commands.remove(&cli_tool.root.uuid).expect("Root command not found");

                let (_, clap_command, mut command_tree) = create_command_tree(&root_command, &commands)?;
                let mut matches = clap_command.get_matches_from(args);

                loop {
                    match &command_tree {
                        CommandTree::NestedCommand { subcommands } => match matches.remove_subcommand() {
                            Some((name, arg_m)) => {
                                command_tree = subcommands[&name].clone();
                                matches = arg_m;
                            },
                            None => bail!("Unexpected error, no subcommand found"),
                        },
                        CommandTree::ScriptCommand { script } => {
                            let mut args = vec![script.to_owned()];

                            if let Some(raw_values) = matches.get_raw("args") {
                                args.extend(raw_values.map(|s| s.to_str().expect("Invalid UTF-8").to_owned()));
                            };

                            run::execute(args).await?;
                        },
                    }
                }
            },
            None => bail!("No command provided, expects @<namespace>/<cli> <args>"),
        }
    }
}

#[derive(Debug, Clone)]
enum CommandTree {
    NestedCommand { subcommands: BTreeMap<String, CommandTree> },
    ScriptCommand { script: String },
}

fn create_command_tree(
    root: &CommandFields,
    map: &BTreeMap<String, CommandFields>,
) -> Result<(String, Command, CommandTree)> {
    match &root.on {
        CommandFieldsOn::NestedCommand(nested) => {
            let mut subcommands = BTreeMap::new();
            let mut command = Command::new(&root.name).arg_required_else_help(true);

            if let Some(description) = &root.description {
                command = command.about(description);
            }

            for subcommand in &nested.subcommands {
                let (name, subcommand, subcommand_tree) = create_command_tree(&map[&subcommand.uuid], map)?;
                command = command.subcommand(subcommand);
                subcommands.insert(name, subcommand_tree);
            }

            Ok((root.name.clone(), command, CommandTree::NestedCommand { subcommands }))
        },
        CommandFieldsOn::ScriptCommand(CommandFieldsOnScriptCommand {
            script: CommandFieldsOnScriptCommandScript { name, namespace },
        }) => {
            let mut command = Command::new(&root.name);

            if let Some(description) = &root.description {
                command = command.about(description);
            }

            // We disable the help flag so it falls through to the script
            command = command.disable_help_flag(true);

            command = command.arg(
                clap::Arg::new("args")
                    .allow_hyphen_values(true)
                    .value_parser(clap::value_parser!(String))
                    .num_args(0..),
            );

            Ok((root.name.clone(), command, CommandTree::ScriptCommand {
                script: match &namespace {
                    Some(namespace) => format!("@{}/{}", namespace.username, name),
                    None => name.to_owned(),
                },
            }))
        },
    }
}
