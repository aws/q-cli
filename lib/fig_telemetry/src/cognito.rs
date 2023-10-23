use aws_credential_types::provider::error::CredentialsError;
use aws_credential_types::{
    provider,
    Credentials,
};
use aws_sdk_cognitoidentity::config::Region;
use aws_sdk_cognitoidentity::error::SdkError;
use aws_sdk_cognitoidentity::operation::get_credentials_for_identity::{
    GetCredentialsForIdentityError,
    GetCredentialsForIdentityOutput,
};
use aws_sdk_cognitoidentity::primitives::{
    DateTime,
    DateTimeFormat,
};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub(crate) struct CognitoPoolId {
    pub(crate) name: &'static str,
    pub(crate) id: &'static str,
}

impl CognitoPoolId {
    #[must_use]
    pub(crate) fn region(&self) -> Region {
        Region::new(self.id.split(':').nth(0).unwrap())
    }
}

// pools from <https://w.amazon.com/bin/view/AWS/DevEx/IDEToolkits/Telemetry/>
pub(crate) const BETA_POOL: CognitoPoolId = CognitoPoolId {
    name: "beta",
    id: "us-east-1:db7bfc9f-8ecd-4fbb-bea7-280c16069a99",
};

pub(crate) const _INTERNAL_PROD: CognitoPoolId = CognitoPoolId {
    name: "internal-prod",
    id: "us-east-1:4037bda8-adbd-4c71-ae5e-88b270261c25",
};

pub(crate) const _EXTERNAL_RROD: CognitoPoolId = CognitoPoolId {
    name: "prod",
    id: "us-east-1:820fd6d1-95c0-4ca4-bffb-3f01d32da842",
};

const CREDENTIALS_KEY: &str = "telemetry-cognito-credentials";

const DATE_TIME_FORMAT: DateTimeFormat = DateTimeFormat::DateTime;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct CredentialsJson {
    pub access_key_id: Option<String>,
    pub secret_key: Option<String>,
    pub session_token: Option<String>,
    pub expiration: Option<String>,
}

pub(crate) async fn get_cognito_credentials_send(
    pool_id: CognitoPoolId,
) -> Result<GetCredentialsForIdentityOutput, SdkError<GetCredentialsForIdentityError>> {
    let region = pool_id.region();
    let conf = aws_sdk_cognitoidentity::Config::builder().region(region).build();
    let client = aws_sdk_cognitoidentity::Client::from_conf(conf);
    client
        .get_credentials_for_identity()
        .identity_id(pool_id.id)
        .send()
        .await
}

pub(crate) async fn get_cognito_credentials(pool_id: CognitoPoolId) -> Result<Credentials, CredentialsError> {
    match fig_settings::state::get_string(CREDENTIALS_KEY).ok().flatten() {
        Some(creds) => {
            let CredentialsJson {
                access_key_id,
                secret_key,
                session_token,
                expiration,
            }: CredentialsJson = serde_json::from_str(&creds).unwrap();

            Ok(Credentials::new(
                access_key_id.unwrap(),
                secret_key.unwrap(),
                session_token,
                expiration
                    .and_then(|s| DateTime::from_str(&s, DATE_TIME_FORMAT).ok())
                    .and_then(|dt| dt.try_into().ok()),
                "",
            ))
        },
        None => {
            let credentials = get_cognito_credentials_send(pool_id)
                .await
                .unwrap()
                .credentials
                .unwrap();

            let _ = fig_settings::state::set_value(
                CREDENTIALS_KEY,
                serde_json::to_value(CredentialsJson {
                    access_key_id: credentials.access_key_id.clone(),
                    secret_key: credentials.secret_key.clone(),
                    session_token: credentials.session_token.clone(),
                    expiration: credentials.expiration.and_then(|t| t.fmt(DATE_TIME_FORMAT).ok()),
                })
                .unwrap(),
            );

            Ok(Credentials::new(
                credentials.access_key_id.unwrap(),
                credentials.secret_key.unwrap(),
                credentials.session_token,
                credentials.expiration.and_then(|dt| dt.try_into().ok()),
                "",
            ))
        },
    }
}

#[derive(Debug)]
pub(crate) struct CognitoProvider {
    pool_id: CognitoPoolId,
}

impl CognitoProvider {
    pub(crate) fn new(pool_id: CognitoPoolId) -> CognitoProvider {
        CognitoProvider { pool_id }
    }
}

impl provider::ProvideCredentials for CognitoProvider {
    fn provide_credentials<'a>(&'a self) -> provider::future::ProvideCredentials<'a>
    where
        Self: 'a,
    {
        provider::future::ProvideCredentials::new(get_cognito_credentials(self.pool_id))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn pools() {
        let all_pools = [BETA_POOL, _INTERNAL_PROD, _EXTERNAL_RROD];
        for id in all_pools {
            let _ = id.region();
            get_cognito_credentials_send(id).await.unwrap();
        }
    }
}
