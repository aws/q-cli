use clap::Parser;
use eyre::{
    bail,
    ContextCompat,
    Result,
};
use fig_request::Request;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{
    Deserialize,
    Serialize,
};
use tokio::process::Command;

use crate::util::choose;

#[derive(Debug, Parser)]
pub struct SshSubcommand {
    /// Host to connect to
    host: String,
    /// Identity to connect with
    #[clap(short = 'a', long = "auth")]
    auth: Option<String>,
    #[clap(long, hide = true)]
    get_identities: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Host {
    nick_name: String,
    ip: String,
    connections: Vec<Connection>,
    #[serde(default)]
    namespace: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "connectionType")]
enum Connection {
    #[serde(rename = "ssh", rename_all = "camelCase")]
    Ssh { port: u16, identity_ids: Vec<String> },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Identity {
    remote_id: u64,
    display_name: String,
    username: String,
    path_to_auth: String,
    namespace: String,
}

impl Connection {
    fn identity_ids(&self) -> &Vec<String> {
        match self {
            Connection::Ssh { identity_ids, .. } => identity_ids,
        }
    }

    fn port(&self) -> u16 {
        match self {
            Connection::Ssh { port, .. } => *port,
        }
    }
}

static HOST_NAMESPACE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?:@([^/]+)/)?(.+)$").expect("Failed compiling host namespace regex"));

impl SshSubcommand {
    pub async fn execute(&self) -> Result<()> {
        let parsed = HOST_NAMESPACE_REGEX
            .captures(&self.host)
            .with_context(|| "invalid host")?;
        let namespace = parsed.get(1).map(|c| c.as_str());
        let host_name = parsed.get(2).unwrap().as_str();
        let hosts: Vec<Host> = if let Some(namespace) = namespace {
            Request::get("/access/hosts")
                .auth()
                .namespace(Some(namespace))
                .deser_json()
                .await?
        } else {
            Request::get("/access/hosts/all").auth().deser_json().await?
        };
        let matching = hosts
            .into_iter()
            .filter(|host| host.nick_name == host_name)
            .collect::<Vec<Host>>();
        let host = match matching.len() {
            0 => {
                bail!("No host matches")
            },
            1 => matching.into_iter().next().unwrap(),
            _ => {
                let chosen = choose(
                    "select host",
                    matching
                        .iter()
                        .map(|host| {
                            if let Some(ns) = &host.namespace {
                                format!("{} ({})", host.nick_name, ns)
                            } else {
                                host.nick_name.clone()
                            }
                        })
                        .collect(),
                )?;
                matching.into_iter().nth(chosen).unwrap()
            },
        };
        let connections = host
            .connections
            .iter()
            .filter(|conn| matches!(conn, Connection::Ssh { .. }))
            .collect::<Vec<&Connection>>();
        if connections.is_empty() {
            bail!("Host has no ssh connections");
        } else if connections.len() > 1 {
            // note(mia): is this ever supposed to happen??
            bail!("Host has multiple ssh connections");
        }
        let connection = connections.into_iter().next().unwrap();

        let identities = connection.identity_ids();
        if identities.is_empty() {
            bail!("Connection has no identities");
        }
        let selected_identity = if let Some(identity) = &self.auth {
            let remote_identities: Vec<Identity> = Request::get("/access/identities").auth().deser_json().await?;
            let name_matches = remote_identities
                .into_iter()
                .filter(|iden| identities.contains(&iden.remote_id.to_string()))
                .filter(|iden| &iden.display_name == identity)
                .collect::<Vec<Identity>>();

            if name_matches.is_empty() {
                bail!("Host has no identity by that name");
            } else if name_matches.len() > 1 {
                let chosen = choose(
                    "select identity",
                    name_matches
                        .iter()
                        .map(|iden| format!("{} ({})", iden.display_name, iden.username))
                        .collect(),
                )?;
                name_matches.into_iter().nth(chosen).unwrap()
            } else {
                name_matches.into_iter().next().unwrap()
            }
        } else {
            let id = identities.iter().next().unwrap();
            let remote_identities: Vec<Identity> = Request::get("/access/identities").auth().deser_json().await?;
            if self.get_identities {
                let user_namespace = fig_api_client::user::account().await?.username;
                println!(
                    "{}",
                    serde_json::to_string_pretty(
                        &remote_identities
                            .into_iter()
                            .filter(|iden| Some(&iden.namespace) == user_namespace.as_ref()
                                || identities.contains(&iden.remote_id.to_string()))
                            .collect::<Vec<Identity>>()
                    )?
                );
                return Ok(());
            }
            let id_matches = remote_identities
                .into_iter()
                .filter(|iden| &iden.remote_id.to_string() == id)
                .collect::<Vec<Identity>>();

            if id_matches.is_empty() {
                bail!("Host has an invalid identity");
            } else if id_matches.len() > 1 {
                // note(mia): this is definitely never supposed to happen
                bail!("Multiple identities with same id!!!");
            }

            id_matches.into_iter().next().unwrap()
        };

        let mut child = Command::new("ssh")
            .arg(format!("{}@{}", selected_identity.username, host.ip))
            .arg("-i")
            .arg(selected_identity.path_to_auth)
            .arg("-p")
            .arg(connection.port().to_string())
            .spawn()?;

        let status = child.wait().await?;

        if !status.success() {
            if let Some(code) = status.code() {
                std::process::exit(code);
            }
            bail!("SSH process was not successful");
        }

        Ok(())
    }
}
