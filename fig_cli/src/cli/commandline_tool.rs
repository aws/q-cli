use clap::Args;
use eyre::{
    bail,
    Result,
};
use fig_api_client::commandline_tool::{
    cached_commandline_tool,
    CommandTree,
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

                let (mut command_tree, refresh_join) =
                    match cached_commandline_tool(namespace.expect("No namespace provided"), name).await {
                        // Ok(Some(_)) is the only case where there could be a refresh_join
                        (Ok(Some(command_tree)), refresh_join) => (command_tree, refresh_join),
                        (Ok(None), _) => bail!("No command found"),
                        (Err(err), _) => bail!(err),
                    };

                let clap_command = create_clap_command(&command_tree);
                let mut matches = clap_command.get_matches_from(args);

                let res = loop {
                    match &command_tree {
                        CommandTree::NestedCommand { subcommands, .. } => match matches.remove_subcommand() {
                            Some((name, arg_m)) => {
                                command_tree = subcommands[&name].clone();
                                matches = arg_m;
                            },
                            None => break Err(eyre::eyre!("Unexpected error, no subcommand found")),
                        },
                        CommandTree::ScriptCommand {
                            script_namespace,
                            script_name,
                            ..
                        } => {
                            let mut args = vec![format!("@{script_namespace}/{script_name}")];

                            if let Some(raw_values) = matches.get_raw("args") {
                                args.extend(raw_values.map(|s| s.to_str().expect("Invalid UTF-8").to_owned()));
                            };

                            break run::execute(args).await;
                        },
                    }
                };

                if let Some(refresh_join) = refresh_join {
                    refresh_join.await.ok();
                }

                res
            },
            None => bail!("No command provided, expects @<namespace>/<cli> <args>"),
        }
    }
}

fn create_clap_command(tree: &CommandTree) -> clap::Command {
    match tree {
        CommandTree::NestedCommand {
            name,
            description,
            subcommands,
            ..
        } => {
            let mut command = clap::Command::new(name).arg_required_else_help(true);

            if let Some(description) = description {
                command = command.about(description);
            }

            for subcommand in subcommands.values() {
                command = command.subcommand(create_clap_command(subcommand));
            }

            command
        },
        CommandTree::ScriptCommand { name, description, .. } => {
            let mut command = clap::Command::new(name);

            if let Some(description) = description {
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

            command
        },
    }
}
