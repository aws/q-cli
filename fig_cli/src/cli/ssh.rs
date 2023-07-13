use std::collections::HashMap;

use clap::Parser;
use eyre::{
    bail,
    ContextCompat,
    Result,
};
use fig_api_client::access::{
    Connection,
    ConnectionType,
    Host,
};
use fig_util::directories;
use once_cell::sync::Lazy;
use regex::Regex;
use tokio::process::Command;
use tracing::warn;

use crate::util::{
    choose_fuzzy,
    dialoguer_theme,
};

#[derive(Debug, Parser, PartialEq, Eq)]
pub struct SshSubcommand {
    /// Host to connect to
    host: Option<String>,
    /// Identity to connect with
    #[arg(short = 'a', long = "auth")]
    auth: Option<String>,
    #[arg(long, hide = true)]
    get_identities: bool,
    /// Ignore saved identities
    #[arg(long, alias = "ignore-saved")]
    ignore_default_identity: bool,
    #[arg(long)]
    remove_default_identity: bool,
}

static HOST_NAMESPACE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?:@([^/]+)/)?(.+)$").expect("Failed compiling host namespace regex"));

impl SshSubcommand {
    pub async fn execute(&self) -> Result<()> {
        if which::which("ssh").is_err() {
            bail!("Couldn't find `ssh`. Please install the OpenSSH client!")
        }

        let mut user = None;

        let saved_identities_path = directories::ssh_saved_identities()?;
        if let Some(parent) = saved_identities_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        if !saved_identities_path.exists() {
            std::fs::write(&saved_identities_path, "")?;
        }

        let (namespace, mut host_name) = match &self.host {
            Some(host) => {
                let parsed = HOST_NAMESPACE_REGEX.captures(host).with_context(|| "invalid host")?;
                let namespace = parsed.get(1).map(|c| c.as_str());
                let host_name = parsed.get(2).unwrap().as_str();
                (namespace.map(|ns| ns.to_string()), Some(host_name))
            },
            None => (None, None),
        };
        let mut hosts = fig_api_client::access::hosts(namespace.clone()).await?;
        if host_name.is_none() || namespace.is_none() {
            let teams = fig_api_client::user::teams().await?;
            let mut tasks = vec![];
            for team in teams {
                tasks.push(tokio::spawn(fig_api_client::access::hosts(Some(team.name))));
            }
            for task in tasks {
                hosts.extend(task.await??);
            }
        }

        let host = loop {
            match host_name {
                Some(host_name_str) => {
                    let filtered = hosts
                        .iter()
                        .filter(|host| host.nick_name == host_name_str)
                        .cloned()
                        .collect::<Vec<Host>>();
                    match filtered.len() {
                        0 => bail!("No hosts found with that name"),
                        1 => break filtered.into_iter().next().unwrap(),
                        _ => {
                            hosts = filtered;
                            host_name = None;
                        },
                    }
                },
                None => {
                    user = Some(fig_api_client::user::account().await?);
                    let idx = choose_fuzzy(
                        "Choose a host to connect to",
                        &hosts
                            .iter()
                            .map(|host| {
                                format!(
                                    "@{}/{}",
                                    host.namespace.as_deref().unwrap_or_else(|| user
                                        .as_ref()
                                        .unwrap()
                                        .username
                                        .as_deref()
                                        .unwrap_or("you")),
                                    host.nick_name,
                                )
                            })
                            .collect::<Vec<String>>(),
                    )?;
                    break hosts.get(idx).cloned().unwrap();
                },
            }
        };

        if self.remove_default_identity {
            let mut entries = read_saved_identities()?;
            entries.remove(&host.remote_id);
            write_saved_identities(entries)?;
            return Ok(());
        }

        let connections = host
            .connections
            .iter()
            .filter(|conn| {
                conn.connection_type == ConnectionType::Ssh || conn.connection_type == ConnectionType::SshJump
            })
            .collect::<Vec<&Connection>>();
        if connections.is_empty() {
            bail!("Host is not configured for ssh");
        } else if connections.len() > 1 {
            bail!("Host has multiple ssh connections, please contact support (hello@fig.io)");
        }
        let connection = connections.into_iter().next().unwrap();

        let mut attempt_save = true;
        let mut identities = Vec::new();
        let selected_identity = if connection.identity_ids.is_empty() && self.auth.is_none() {
            None
        } else {
            identities.extend(fig_api_client::access::identities(host.namespace.clone()).await?);
            if self.auth.is_none() && connection.default_identity_id.is_some() {
                let default = connection.default_identity_id.unwrap();
                if identities.iter().any(|iden| iden.remote_id == default) {
                    identities.retain(|iden| iden.remote_id == default);
                }
            } else {
                identities.retain(|iden| connection.identity_ids.contains(&iden.remote_id));
            }

            if host.namespace.is_some() {
                if user.is_none() {
                    user = Some(fig_api_client::user::account().await?);
                }
                if user.as_ref().unwrap().username != host.namespace {
                    identities.extend(fig_api_client::access::identities(None).await?);
                }
            }

            if let Some(auth) = &self.auth {
                let auth_lower = auth.to_lowercase();
                if !identities.iter().any(|x| x.display_name.to_lowercase() == auth_lower) {
                    bail!("Identity {auth} not found");
                }
                identities.retain(|x| x.display_name.to_lowercase() == auth_lower);
            }

            if self.get_identities {
                println!("{}", serde_json::to_string_pretty(&identities)?);
                return Ok(());
            }
            if identities.len() > 1 && !self.ignore_default_identity {
                let entries = read_saved_identities()?;
                if let Some(Some(iden)) = entries.get(&host.remote_id) {
                    if identities.iter().any(|check| check.remote_id == *iden) {
                        identities.retain(|check| check.remote_id == *iden);
                        attempt_save = false;
                    }
                }
            }

            identities.dedup_by_key(|iden| iden.remote_id);

            match identities.len() {
                0 => {
                    warn!("No identities found!");
                    None
                },
                1 => {
                    attempt_save = false;
                    identities.first()
                },
                _ => {
                    if user.is_none() {
                        user = Some(fig_api_client::user::account().await?);
                    }
                    let idx = choose_fuzzy(
                        "Choose an identity to connect with",
                        &identities
                            .iter()
                            .map(|iden| {
                                format!(
                                    "@{}/{}",
                                    iden.namespace.as_deref().unwrap_or_else(|| user
                                        .as_ref()
                                        .unwrap()
                                        .username
                                        .as_deref()
                                        .unwrap_or("you")),
                                    iden.display_name,
                                )
                            })
                            .collect::<Vec<String>>(),
                    )?;
                    identities.get(idx)
                },
            }
        };

        if attempt_save {
            if let Some(selected_identity) = selected_identity {
                let mut entries = read_saved_identities()?;
                let host_entry = if self.ignore_default_identity {
                    None
                } else {
                    entries.get(&host.remote_id)
                };
                match host_entry {
                    None | Some(Some(_)) => {
                        let should_add_new = dialoguer::Confirm::with_theme(&dialoguer_theme())
                            .with_prompt("Would you like to save this identity?")
                            .interact()?;
                        entries.insert(
                            host.remote_id,
                            if should_add_new {
                                Some(selected_identity.remote_id)
                            } else {
                                None
                            },
                        );
                    },
                    _ => {},
                }
                write_saved_identities(entries)?;
            }
        }

        let ssh_string =
            fig_api_client::access::ssh_string(host.remote_id, selected_identity.as_ref().map(|iden| iden.remote_id))
                .await?;

        let mut command = None;

        if let Some(identity) = selected_identity {
            if identity.authentication_type == "password" {
                let expect_bin = match which::which("expect") {
                    Ok(expect_bin) => expect_bin,
                    Err(_) => {
                        eyre::bail!("You need to install expect to use password input");
                    },
                };
                let password = identity.password.clone().unwrap_or_default();
                let mut expect_command = Command::new(expect_bin);
                expect_command.arg("-c");
                expect_command.arg(format!(r#"spawn {ssh_string}; expect "yes/no" {{ send "yes\n"; expect "*?assword:"; send "{password}\n"; }} "*?assword" {{ send "{password}\n"; }}; interact"#));
                command = Some(expect_command);
            }
        }

        let mut command = match command {
            Some(command) => command,
            None => {
                let mut parts = shlex::split(&ssh_string)
                    .context("got no built ssh string from api")?
                    .into_iter();

                let mut command = Command::new(parts.next().context("didn't get root command")?);

                for arg in parts {
                    command.arg(arg);
                }

                command
            },
        };

        println!(
            "Connecting to {}{}{}",
            host.namespace.map(|ns| format!("@{ns}/")).unwrap_or_default(),
            host.nick_name,
            selected_identity
                .as_ref()
                .map(|iden| format!(
                    " with identity {}{}",
                    iden.namespace.as_ref().map(|ns| format!("@{ns}/")).unwrap_or_default(),
                    iden.display_name
                ))
                .unwrap_or_default()
        );

        let status = command.spawn()?.wait().await?;

        if !status.success() {
            if let Some(code) = status.code() {
                std::process::exit(code);
            }
            bail!("SSH process was not successful");
        }

        Ok(())
    }
}

fn read_saved_identities() -> eyre::Result<HashMap<u64, Option<u64>>> {
    let saved = std::fs::read_to_string(directories::ssh_saved_identities()?)?;
    Ok(HashMap::from_iter(
        saved
            .lines()
            .filter_map(|l| l.split_once('='))
            .filter_map(|(h, i)| Some((h.parse::<u64>().ok()?, i.parse::<u64>().ok()))),
    ))
}

fn write_saved_identities(entries: HashMap<u64, Option<u64>>) -> eyre::Result<()> {
    let updated = entries
        .iter()
        .map(|(host, iden)| format!("{host}={}\n", iden.map(|x| x.to_string()).unwrap_or_default()))
        .collect::<String>();
    std::fs::write(directories::ssh_saved_identities()?, updated)?;

    Ok(())
}
