use amzn_toolkit_telemetry::config::AppName;
use aws_credential_types::provider::error::CredentialsError;
use aws_credential_types::{
    provider,
    Credentials,
};
use aws_sdk_cognitoidentity::config::Region;
use aws_sdk_cognitoidentity::primitives::{
    DateTime,
    DateTimeFormat,
};

use crate::APP_NAME;

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

pub(crate) async fn get_cognito_credentials_send(pool_id: CognitoPoolId) -> Result<Credentials, CredentialsError> {
    let region = pool_id.region();
    let conf = aws_sdk_cognitoidentity::Config::builder()
        .region(region)
        .app_name(AppName::new(APP_NAME).unwrap())
        .build();
    let client = aws_sdk_cognitoidentity::Client::from_conf(conf);

    let identity_id = client
        .get_id()
        .identity_pool_id(pool_id.id)
        .send()
        .await
        .map_err(CredentialsError::provider_error)?
        .identity_id
        .ok_or(CredentialsError::provider_error("no identity_id from get_id"))?;
    
    let credentials = client
        .get_credentials_for_identity()
        .identity_id(identity_id)
        .send()
        .await
        .map_err(CredentialsError::provider_error)?
        .credentials
        .ok_or(CredentialsError::provider_error("no credentials from get_credentials_for_identity"))?;

    if let Ok(json) = serde_json::to_value(CredentialsJson {
        access_key_id: credentials.access_key_id.clone(),
        secret_key: credentials.secret_key.clone(),
        session_token: credentials.session_token.clone(),
        expiration: credentials.expiration.and_then(|t| t.fmt(DATE_TIME_FORMAT).ok()),
    }) {
        fig_settings::state::set_value(CREDENTIALS_KEY, json).ok();
    }

    let Some(access_key_id) = credentials.access_key_id else {
        return Err(CredentialsError::provider_error("access key id not found"));
    };

    let Some(secret_key) = credentials.secret_key else {
        return Err(CredentialsError::provider_error("secret access key not found"));
    };

    Ok(Credentials::new(
        access_key_id,
        secret_key,
        credentials.session_token,
        credentials.expiration.and_then(|dt| dt.try_into().ok()),
        "",
    ))
}

pub(crate) async fn get_cognito_credentials(pool_id: CognitoPoolId) -> Result<Credentials, CredentialsError> {
    match fig_settings::state::get_string(CREDENTIALS_KEY).ok().flatten() {
        Some(creds) => {
            let CredentialsJson {
                access_key_id,
                secret_key,
                session_token,
                expiration,
            }: CredentialsJson = serde_json::from_str(&creds).map_err(CredentialsError::provider_error)?;

            let Some(access_key_id) = access_key_id else {
                return get_cognito_credentials_send(pool_id).await;
            };

            let Some(secret_key) = secret_key else {
                return get_cognito_credentials_send(pool_id).await;
            };

            Ok(Credentials::new(
                access_key_id,
                secret_key,
                session_token,
                expiration
                    .and_then(|s| DateTime::from_str(&s, DATE_TIME_FORMAT).ok())
                    .and_then(|dt| dt.try_into().ok()),
                "",
            ))
        },
        None => get_cognito_credentials_send(pool_id).await,
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
