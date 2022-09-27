use serde::{
    Deserialize,
    Serialize,
};

use crate::util::{
    string_as_option_u64,
    string_as_vec_u64,
};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Host {
    pub nick_name: String,
    pub ip: String,
    pub remote_id: u64,
    pub tags: Vec<String>,
    pub description: String,
    pub connections: Vec<Connection>,
    #[serde(default)]
    pub namespace: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    pub remote_id: u64,
    pub connection_type: ConnectionType,
    pub port: u16,
    #[serde(deserialize_with = "string_as_option_u64", default)]
    pub default_identity_id: Option<u64>,
    #[serde(deserialize_with = "string_as_vec_u64")]
    pub identity_ids: Vec<u64>,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum ConnectionType {
    #[serde(rename = "ssh")]
    Ssh,
}

pub async fn hosts(namespace: Option<String>) -> fig_request::Result<Vec<Host>> {
    Ok(fig_request::Request::get("/access/hosts")
        .auth()
        .namespace(namespace.clone())
        .deser_json::<Vec<Host>>()
        .await?
        .into_iter()
        .map(|mut host| {
            host.namespace = namespace.clone();
            host
        })
        .collect())
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    pub username: String,
    pub display_name: String,
    pub path_to_auth: Option<String>,
    pub remote_id: u64,
    pub namespace: Option<String>,
    pub private_key: Option<String>,
    pub authentication_type: String,
}

pub async fn identities(namespace: Option<String>) -> fig_request::Result<Vec<Identity>> {
    Ok(fig_request::Request::get("/access/identities")
        .auth()
        .namespace(namespace.clone())
        .deser_json::<Vec<Identity>>()
        .await?
        .into_iter()
        .map(|mut host| {
            host.namespace = namespace.clone();
            host
        })
        .collect())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SshStringRequest<'a> {
    authentication_type: Option<&'a str>,
    path_to_auth: Option<&'a str>,
    identity_remote_id: Option<u64>,
    username: Option<&'a str>,
    hostname: &'a str,
    port: u16,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SshStringResponse {
    ssh_string: String,
}

pub async fn ssh_string(
    host: &Host,
    connection: &Connection,
    identity: &Option<Identity>,
) -> fig_request::Result<String> {
    Ok(fig_request::Request::get("/access/ssh_string")
        .auth()
        .body(SshStringRequest {
            authentication_type: identity.as_ref().map(|iden| iden.authentication_type.as_ref()),
            path_to_auth: identity.as_ref().and_then(|iden| iden.path_to_auth.as_deref()),
            identity_remote_id: identity.as_ref().map(|iden| iden.remote_id),
            username: identity.as_ref().map(|iden| iden.username.as_ref()),
            hostname: &host.ip,
            port: connection.port,
        })
        .deser_json::<SshStringResponse>()
        .await?
        .ssh_string)
}
