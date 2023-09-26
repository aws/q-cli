use std::error::Error;

use aws_sdk_ssooidc::client::Client;
use aws_sdk_ssooidc::error::SdkError;
use aws_types::region::Region;
use once_cell::sync::Lazy;
use tokio::sync::{
    Mutex,
    MutexGuard,
};
use tracing::error;

pub const CLIENT_NAME: &str = "CodeWhisperer for Terminal";

pub const OIDC_BUILDER_ID_ENDPOINT: &str = "https://oidc.us-east-1.amazonaws.com";
pub const OIDC_BUILDER_ID_REGION: Region = Region::from_static("us-east-1");

pub const SCOPES: &[&str] = &["codewhisperer:completions"];
pub const CLIENT_TYPE: &str = "public";
pub const START_URL: &str = "https://view.awsapps.com/start";
pub const GRANT_TYPE: &str = "urn:ietf:params:oauth:grant-type:device_code";

const ACCESS_TOKEN_KEY: &str = "codewhisper:builder-id:access-token";

static BUILDER_ID_TOKEN: Mutex<Option<String>> = Mutex::const_new(None);

pub async fn builder_id_token() -> MutexGuard<'static, Option<String>> {
    let mut builder_id_token = BUILDER_ID_TOKEN.lock().await;

    if builder_id_token.is_none() {
        // try to grab the secret from the secret store
        match crate::secret_store::get_secret(ACCESS_TOKEN_KEY) {
            Ok(Some(access_token)) => {
                *builder_id_token = Some(access_token);
            },
            Ok(None) => {},
            Err(err) => {
                error!("Error getting builder id token from keychain: {}", err);
            },
        };
    }

    builder_id_token
}

static CLIENT: Lazy<Client> = Lazy::new(|| {
    let sdk_config = aws_types::SdkConfig::builder()
        .endpoint_url(OIDC_BUILDER_ID_ENDPOINT)
        .region(OIDC_BUILDER_ID_REGION)
        .build();
    Client::new(&sdk_config)
});

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BuilderIdInit {
    pub code: String,
    pub url: String,
    pub device_code: String,
    pub dynamic_client_id: String,
    pub dynamic_client_secret: String,
    pub expires_in: i32,
    pub interval: i32,
}

/// Init a builder id request
pub async fn builder_id_init() -> Result<BuilderIdInit, aws_sdk_ssooidc::Error> {
    let mut register = CLIENT
        .register_client()
        .client_name(CLIENT_NAME)
        .client_type(CLIENT_TYPE);
    for scope in SCOPES {
        register = register.scopes(*scope);
    }
    let register_res = register.send().await?;

    let dynamic_client_id = register_res.client_id.unwrap_or_default();
    let dynamic_client_secret = register_res.client_secret.unwrap_or_default();

    let device_auth_res = CLIENT
        .start_device_authorization()
        .client_id(&dynamic_client_id)
        .client_secret(&dynamic_client_secret)
        .start_url(START_URL)
        .send()
        .await?;

    let url = device_auth_res.verification_uri_complete.unwrap_or_default();
    let code = device_auth_res.user_code.unwrap_or_default();
    let device_code = device_auth_res.device_code.unwrap_or_default();
    let expires_in = device_auth_res.expires_in;
    let interval = device_auth_res.interval;

    Ok(BuilderIdInit {
        code,
        url,
        device_code,
        dynamic_client_id,
        dynamic_client_secret,
        expires_in,
        interval,
    })
}

pub enum BuilderIdPollStatus<E: Error> {
    Pending,
    Complete,
    Error(E),
}

pub async fn builder_id_poll(
    device_code: String,
    dynamic_client_id: String,
    dynamic_client_secret: String,
) -> BuilderIdPollStatus<aws_sdk_ssooidc::Error> {
    match CLIENT
        .create_token()
        .grant_type(GRANT_TYPE)
        .device_code(device_code)
        .client_id(dynamic_client_id)
        .client_secret(dynamic_client_secret)
        .send()
        .await
    {
        Ok(token_res) => {
            let access_token = token_res.access_token().unwrap();

            // we hold this across the set_secret to ensure they are in sync
            let mut secret = BUILDER_ID_TOKEN.lock().await;
            if let Err(err) = crate::secret_store::set_secret(ACCESS_TOKEN_KEY, access_token) {
                error!(?err, "Failed to store builder id access token");
            };
            *secret = Some(access_token.to_owned());

            BuilderIdPollStatus::Complete
        },
        Err(SdkError::ServiceError(service_error)) if service_error.err().is_authorization_pending_exception() => {
            BuilderIdPollStatus::Pending
        },
        Err(err) => BuilderIdPollStatus::Error(err.into()),
    }
}
