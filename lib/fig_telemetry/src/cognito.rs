use std::time::SystemTime;

use aws_sdk_cognitoidentity::config::Region;
use aws_sdk_cognitoidentity::primitives::{
    DateTime,
    DateTimeFormat,
};
use aws_sdk_cognitoidentity::types::Credentials;

pub(crate) struct CognitoPoolId(pub(crate) &'static str);

impl CognitoPoolId {
    #[must_use]
    pub(crate) fn region(&self) -> Region {
        Region::new(self.0.split(':').nth(0).unwrap())
    }
}

impl Into<String> for CognitoPoolId {
    fn into(self) -> String {
        self.0.to_owned()
    }
}

// pools from <https://w.amazon.com/bin/view/AWS/DevEx/IDEToolkits/Telemetry/>
const BETA_POOL: CognitoPoolId = CognitoPoolId("us-east-1:db7bfc9f-8ecd-4fbb-bea7-280c16069a99");
const INTERNAL_PROD: CognitoPoolId = CognitoPoolId("us-east-1:4037bda8-adbd-4c71-ae5e-88b270261c25");
const EXTERNAL_RROD: CognitoPoolId = CognitoPoolId("us-east-1:820fd6d1-95c0-4ca4-bffb-3f01d32da842");

const CREDENTIALS_KEY: &str = "telemetry-cognito-credentials";

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct CredentialsJson {
    pub access_key_id: Option<String>,
    pub secret_key: Option<String>,
    pub session_token: Option<String>,
    pub expiration: Option<String>,
}

async fn get_cognito_credentials() -> Result<Credentials, ()> {
    match fig_settings::state::get_string(CREDENTIALS_KEY).ok().flatten() {
        Some(creds) => {
            let CredentialsJson {
                access_key_id,
                secret_key,
                session_token,
                expiration,
            }: CredentialsJson = serde_json::from_str(&creds).unwrap();

            Ok(Credentials::builder()
                .set_access_key_id(access_key_id)
                .set_secret_key(secret_key)
                .set_session_token(session_token)
                .set_expiration(expiration.and_then(|t| DateTime::from_str(&t, DateTimeFormat::DateTime).ok()))
                .build())
        },
        None => {
            let region = BETA_POOL.region();
            let conf = aws_sdk_cognitoidentity::Config::builder().region(region).build();
            let client = aws_sdk_cognitoidentity::Client::from_conf(conf);
            let creds = client
                .get_credentials_for_identity()
                .identity_id(BETA_POOL)
                .send()
                .await
                .unwrap();

            let a = creds.credentials.unwrap();

            let _ = fig_settings::state::set_value(
                CREDENTIALS_KEY,
                serde_json::to_value(&CredentialsJson {
                    access_key_id: a.access_key_id.clone(),
                    secret_key: a.secret_key.clone(),
                    session_token: a.session_token.clone(),
                    expiration: a.expiration.and_then(|t| t.fmt(DateTimeFormat::DateTime).ok()),
                })
                .unwrap(),
            );

            Ok(a)
        },
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn pools() {
        let all_pools = [BETA_POOL, INTERNAL_PROD, EXTERNAL_RROD];
        for pool in all_pools {
            let _ = pool.region();
        }
    }
}
