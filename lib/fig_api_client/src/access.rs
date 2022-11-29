use serde::{
    Deserialize,
    Serialize,
};

use crate::util::{
    string_as_option_u64,
    string_as_vec_u64,
};

#[derive(Deserialize, Debug, Clone)]
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

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    pub remote_id: u64,
    pub connection_type: ConnectionType,
    pub port: u16,
    #[serde(deserialize_with = "string_as_option_u64", default)]
    pub default_identity_id: Option<u64>,
    #[serde(deserialize_with = "string_as_vec_u64")]
    pub identity_ids: Vec<u64>,
    pub port_forwards: Vec<PortForward>,
    #[serde(deserialize_with = "string_as_option_u64", default)]
    pub remote_host_id: Option<u64>,
    #[serde(deserialize_with = "string_as_option_u64", default)]
    pub remote_host_identity_id: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PortForward {
    pub remote_id: u64,
    pub forwarded_ip: String,
    pub forwarded_port: u16,
    pub ip: String,
    pub port: u16,
    pub port_forward_type: PortForwardType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum PortForwardType {
    Local,
    Remote,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionType {
    Ssh,
    SshJump,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    pub username: String,
    pub display_name: String,
    pub path_to_auth: Option<String>,
    pub remote_id: u64,
    pub namespace: Option<String>,
    pub private_key: Option<String>,
    pub authentication_type: String,
    pub password: Option<String>,
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
struct SshStringRequest {
    host_id: u64,
    identity_id: Option<u64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SshStringResponse {
    ssh_string: String,
}

pub async fn ssh_string(host_id: u64, identity_id: Option<u64>) -> fig_request::Result<String> {
    Ok(fig_request::Request::get("/access/v2/ssh_string")
        .auth()
        .body(SshStringRequest { host_id, identity_id })
        .deser_json::<SshStringResponse>()
        .await?
        .ssh_string)
}
