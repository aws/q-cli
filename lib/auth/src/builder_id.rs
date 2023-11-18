//! # Builder ID
//!
//!  SSO flow (RFC: <https://tools.ietf.org/html/rfc8628>)
//!    1. Get a client id (SSO-OIDC identifier, formatted per RFC6749).
//!       - Code: [DeviceRegistration::register]
//!          - Calls [Client::register_client]
//!       - RETURNS: [RegisterClientResponse]
//!       - Client registration is valid for potentially months and creates state server-side, so
//!         the client SHOULD cache them to disk.
//!    2. Start device authorization.
//!       - Code: [start_device_authorization]
//!          - Calls [Client::start_device_authorization]
//!       - RETURNS (RFC: <https://tools.ietf.org/html/rfc8628#section-3.2>):
//!         [StartDeviceAuthorizationResponse]
//!    3. Poll for the access token
//!       - Code: [poll_create_token]
//!          - Calls [Client::create_token]
//!       - RETURNS: [CreateTokenResponse]
//!    4. (Repeat) Tokens SHOULD be refreshed if expired and a refresh token is available.
//!        - Code: [refresh_token]
//!          - Calls [Client::create_token]
//!        - RETURNS: [CreateTokenResponse]

use aws_sdk_ssooidc::client::Client;
use aws_sdk_ssooidc::config::retry::RetryConfig;
use aws_sdk_ssooidc::config::{
    ConfigBag,
    SharedAsyncSleep,
};
use aws_sdk_ssooidc::error::SdkError;
use aws_sdk_ssooidc::operation::create_token::CreateTokenOutput;
use aws_smithy_async::future::now_or_later::NowOrLater;
use aws_smithy_async::rt::sleep::TokioSleep;
use aws_smithy_runtime_api::client::identity::http::Token;
use aws_smithy_runtime_api::client::identity::{
    Identity,
    IdentityResolver,
};
use aws_smithy_runtime_api::client::orchestrator::Future;
use aws_types::app_name::AppName;
use aws_types::region::Region;
use once_cell::sync::Lazy;
use time::OffsetDateTime;
use tracing::error;

use crate::secret_store::{
    Secret,
    SecretStore,
};
use crate::{
    Error,
    Result,
};

const CLIENT_NAME: &str = "CodeWhisperer";

const OIDC_BUILDER_ID_REGION: Region = Region::from_static("us-east-1");

const SCOPES: &[&str] = &[
    "sso:account:access",
    "codewhisperer:completions",
    "codewhisperer:analysis",
];
const CLIENT_TYPE: &str = "public";
const START_URL: &str = "https://view.awsapps.com/start";

const DEVICE_GRANT_TYPE: &str = "urn:ietf:params:oauth:grant-type:device_code";
const REFRESH_GRANT_TYPE: &str = "refresh_token";

static APP_NAME: Lazy<AppName> = Lazy::new(|| AppName::new("codewhisperer-terminal").unwrap());

/// Indicates if an expiration time has passed, there is a small 1 min window that is removed
/// so the token will not expire in transit
fn is_expired(expiration_time: &OffsetDateTime) -> bool {
    let now = time::OffsetDateTime::now_utc();
    &(now + time::Duration::minutes(1)) > expiration_time
}

fn oidc_url(region: &Region) -> String {
    format!("https://oidc.{region}.amazonaws.com")
}

fn client(region: Region) -> Client {
    let retry_config = RetryConfig::standard().with_max_attempts(3);
    let sdk_config = aws_types::SdkConfig::builder()
        .endpoint_url(oidc_url(&region))
        .region(region)
        .retry_config(retry_config)
        .sleep_impl(SharedAsyncSleep::new(TokioSleep::new()))
        .app_name(APP_NAME.clone())
        .build();
    Client::new(&sdk_config)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeviceRegistration {
    pub client_id: String,
    pub client_secret: Secret,
    #[serde(with = "time::serde::rfc3339::option")]
    pub client_secret_expires_at: Option<time::OffsetDateTime>,
    pub region: String,
}

impl DeviceRegistration {
    const SECRET_KEY: &'static str = "codewhisperer:odic:device-registration";

    async fn load_from_secret_store(secret_store: &SecretStore, region: &Region) -> Result<Option<Self>> {
        let device_registration = secret_store.get(Self::SECRET_KEY).await?;

        if let Some(device_registration) = device_registration {
            // check that the data is not expired, assume it is invalid if not present
            let device_registration: Self = serde_json::from_str(&device_registration.0)?;

            if let Some(client_secret_expires_at) = device_registration.client_secret_expires_at {
                if !is_expired(&client_secret_expires_at) && device_registration.region == region.as_ref() {
                    return Ok(Some(device_registration));
                }
            }
        }

        // delete the data if its expired or invalid
        if let Err(err) = secret_store.delete(Self::SECRET_KEY).await {
            error!(?err, "Failed to delete device registration from keychain");
        }

        Ok(None)
    }

    /// Register the client with OIDC and cache the response for the expiration period
    pub async fn register(client: &Client, secret_store: &SecretStore, region: &Region) -> Result<Self> {
        match Self::load_from_secret_store(secret_store, region).await {
            Ok(Some(device_registration)) => return Ok(device_registration),
            Ok(None) => {},
            Err(err) => {
                error!(?err, "Failed to read device registration from keychain");
            },
        };

        let mut register = client
            .register_client()
            .client_name(CLIENT_NAME)
            .client_type(CLIENT_TYPE);
        for scope in SCOPES {
            register = register.scopes(*scope);
        }
        let output = register.send().await?;

        let device_registration = Self {
            client_id: output.client_id.unwrap_or_default(),
            client_secret: output.client_secret.unwrap_or_default().into(),
            client_secret_expires_at: time::OffsetDateTime::from_unix_timestamp(output.client_secret_expires_at).ok(),
            region: region.to_string(),
        };

        if let Err(err) = secret_store
            .set(Self::SECRET_KEY, &serde_json::to_string(&device_registration)?)
            .await
        {
            error!(?err, "Failed to write device registration to keychain");
        }

        Ok(device_registration)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StartDeviceAuthorizationResponse {
    /// Device verification code.
    pub device_code: String,
    /// User verification code.
    pub user_code: String,
    /// Verification URI on the authorization server.
    pub verification_uri: String,
    /// User verification URI on the authorization server.
    pub verification_uri_complete: String,
    /// Lifetime (seconds) of `device_code` and `user_code`.
    pub expires_in: i32,
    /// Minimum time (seconds) the client SHOULD wait between polling intervals.
    pub interval: i32,
    pub region: String,
    pub start_url: String,
}

/// Init a builder id request
pub async fn start_device_authorization(
    secret_store: &SecretStore,
    start_url: Option<String>,
    region: Option<String>,
) -> Result<StartDeviceAuthorizationResponse> {
    let region = region.clone().map_or(OIDC_BUILDER_ID_REGION, Region::new);
    let client = client(region.clone());

    let DeviceRegistration {
        client_id,
        client_secret,
        ..
    } = DeviceRegistration::register(&client, secret_store, &region).await?;

    let output = client
        .start_device_authorization()
        .client_id(&client_id)
        .client_secret(&client_secret.0)
        .start_url(start_url.as_deref().unwrap_or(START_URL))
        .send()
        .await?;

    Ok(StartDeviceAuthorizationResponse {
        device_code: output.device_code.unwrap_or_default(),
        user_code: output.user_code.unwrap_or_default(),
        verification_uri: output.verification_uri.unwrap_or_default(),
        verification_uri_complete: output.verification_uri_complete.unwrap_or_default(),
        expires_in: output.expires_in,
        interval: output.interval,
        region: region.to_string(),
        start_url: start_url.unwrap_or_else(|| START_URL.to_owned()),
    })
}

pub enum TokenType {
    BuilderId,
    IamIdentityCenter,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BuilderIdToken {
    pub access_token: Secret,
    #[serde(with = "time::serde::rfc3339")]
    pub expires_at: time::OffsetDateTime,
    pub refresh_token: Option<Secret>,
    pub region: Option<String>,
    pub start_url: Option<String>,
}

impl BuilderIdToken {
    const SECRET_KEY: &'static str = "codewhisperer:odic:token";

    /// Load the token from the keychain, refresh the token if it is expired and return it
    pub async fn load(secret_store: &SecretStore) -> Result<Option<Self>> {
        match secret_store.get(Self::SECRET_KEY).await {
            Ok(Some(secret)) => {
                let token: Option<Self> = serde_json::from_str(&secret.0)?;
                match token {
                    Some(token) => {
                        let region = token.region.clone().map_or(OIDC_BUILDER_ID_REGION, Region::new);

                        let client = client(region.clone());
                        // if token is expired try to refresh
                        if token.is_expired() {
                            token.refresh_token(&client, secret_store, &region).await
                        } else {
                            Ok(Some(token))
                        }
                    },
                    None => Ok(None),
                }
            },
            Ok(None) => Ok(None),
            Err(err) => {
                error!(%err, "Error getting builder id token from keychain");
                Err(err)
            },
        }
    }

    /// Refresh the access token
    pub async fn refresh_token(
        &self,
        client: &Client,
        secret_store: &SecretStore,
        region: &Region,
    ) -> Result<Option<Self>> {
        // TODO: add telem on error https://github.com/aws/aws-toolkit-vscode/blob/df9fc7315b37844a09b7f60b49c604fa267a88ab/src/auth/sso/ssoAccessTokenProvider.ts#L135-L143

        let DeviceRegistration {
            client_id,
            client_secret,
            ..
        } = DeviceRegistration::register(client, secret_store, region).await?;

        let Some(refresh_token) = &self.refresh_token else {
            // if the token is expired and has no refresh token, delete it
            if let Err(err) = self.delete(secret_store).await {
                error!(?err, "Failed to delete builder id token");
            }

            return Ok(None);
        };

        match client
            .create_token()
            .client_id(client_id)
            .client_secret(client_secret.0)
            .refresh_token(&refresh_token.0)
            .grant_type(REFRESH_GRANT_TYPE)
            .send()
            .await
        {
            Ok(output) => {
                let token: BuilderIdToken = Self::from_output(output, region.clone(), self.start_url.clone());

                if let Err(err) = token.save(secret_store).await {
                    error!(?err, "Failed to store builder id access token");
                };

                Ok(Some(token))
            },
            Err(err) => {
                error!(?err, "Failed to refresh builder id access token");

                // if the error is the client's fault, clear the token
                if let SdkError::ServiceError(service_err) = &err {
                    if !service_err.err().is_slow_down_exception() {
                        if let Err(err) = self.delete(secret_store).await {
                            error!(?err, "Failed to delete builder id token");
                        }
                    }
                }

                Err(err.into())
            },
        }
    }

    /// If the time has passed the `expires_at` time
    ///
    /// The token is marked as expired 1 min before it actually does to account for the potential a
    /// token expires while in transit
    pub fn is_expired(&self) -> bool {
        is_expired(&self.expires_at)
    }

    /// Save the token to the keychain
    pub async fn save(&self, secret_store: &SecretStore) -> Result<()> {
        secret_store
            .set(Self::SECRET_KEY, &serde_json::to_string(self)?)
            .await?;
        Ok(())
    }

    /// Delete the token from the keychain
    pub async fn delete(&self, secret_store: &SecretStore) -> Result<()> {
        secret_store.delete(Self::SECRET_KEY).await?;
        Ok(())
    }

    fn from_output(output: CreateTokenOutput, region: Region, start_url: Option<String>) -> Self {
        Self {
            access_token: output.access_token.unwrap_or_default().into(),
            expires_at: time::OffsetDateTime::now_utc() + time::Duration::seconds(output.expires_in as i64),
            refresh_token: output.refresh_token.map(|t| t.into()),
            region: Some(region.to_string()),
            start_url,
        }
    }

    pub fn token_type(&self) -> TokenType {
        match &self.start_url {
            Some(url) if url == START_URL => TokenType::BuilderId,
            None => TokenType::BuilderId,
            Some(_) => TokenType::IamIdentityCenter,
        }
    }
}

pub enum PollCreateToken {
    Pending,
    Complete(BuilderIdToken),
    Error(Error),
}

/// Poll for the create token response
pub async fn poll_create_token(
    secret_store: &SecretStore,
    device_code: String,
    start_url: Option<String>,
    region: Option<String>,
) -> PollCreateToken {
    let region = region.clone().map_or(OIDC_BUILDER_ID_REGION, Region::new);
    let client = client(region.clone());

    let DeviceRegistration {
        client_id,
        client_secret,
        ..
    } = match DeviceRegistration::register(&client, secret_store, &region).await {
        Ok(res) => res,
        Err(err) => {
            return PollCreateToken::Error(err);
        },
    };

    match client
        .create_token()
        .grant_type(DEVICE_GRANT_TYPE)
        .device_code(device_code)
        .client_id(client_id)
        .client_secret(client_secret.0)
        .send()
        .await
    {
        Ok(output) => {
            let token: BuilderIdToken = BuilderIdToken::from_output(output, region, start_url);

            if let Err(err) = token.save(secret_store).await {
                error!(?err, "Failed to store builder id token");
            };

            fig_telemetry::send_user_logged_in().await;

            PollCreateToken::Complete(token)
        },
        Err(SdkError::ServiceError(service_error)) if service_error.err().is_authorization_pending_exception() => {
            PollCreateToken::Pending
        },
        Err(err) => {
            error!(?err, "Failed to poll for builder id token");
            PollCreateToken::Error(err.into())
        },
    }
}

pub async fn is_logged_in() -> bool {
    let Ok(secret_store) = SecretStore::load().await else {
        return false;
    };

    matches!(BuilderIdToken::load(&secret_store).await, Ok(Some(_)))
}

pub async fn logout() -> Result<()> {
    let Ok(secret_store) = SecretStore::load().await else {
        return Ok(());
    };

    tokio::try_join!(
        secret_store.delete(BuilderIdToken::SECRET_KEY),
        secret_store.delete(DeviceRegistration::SECRET_KEY),
    )?;

    Ok(())
}

#[derive(Debug, Clone)]
pub struct BearerResolver;

impl IdentityResolver for BearerResolver {
    fn resolve_identity(&self, _config_bag: &ConfigBag) -> Future<Identity> {
        NowOrLater::new(Box::pin(async {
            let secret_store = SecretStore::load().await?;
            let token = BuilderIdToken::load(&secret_store).await?;
            match token {
                Some(token) => Ok(Identity::new(
                    Token::new(token.access_token.0, Some(token.expires_at.into())),
                    Some(token.expires_at.into()),
                )),
                None => Err(Error::NoToken.into()),
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const US_EAST_1: Region = Region::from_static("us-east-1");
    const US_WEST_2: Region = Region::from_static("us-west-2");

    #[test]
    fn test_app_name() {
        println!("{:?}", *APP_NAME);
    }

    #[test]
    fn test_client() {
        println!("{:?}", client(US_EAST_1));
        println!("{:?}", client(US_WEST_2));
    }

    #[test]
    fn oidc_url_snapshot() {
        insta::assert_snapshot!(oidc_url(&US_EAST_1), @"https://oidc.us-east-1.amazonaws.com");
        insta::assert_snapshot!(oidc_url(&US_WEST_2), @"https://oidc.us-west-2.amazonaws.com");
    }

    #[ignore = "login flow"]
    #[tokio::test]
    async fn test_login() {
        let start_url = Some("https://d-90678a77a2.awsapps.com/start".into());
        let secret_store = SecretStore::load().await.unwrap();
        let res = start_device_authorization(&secret_store, start_url.clone(), None)
            .await
            .unwrap();
        println!("{:?}", res);
        loop {
            match poll_create_token(&secret_store, res.device_code.clone(), start_url.clone(), None).await {
                PollCreateToken::Pending => {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                },
                PollCreateToken::Complete(token) => {
                    println!("{:?}", token);
                    break;
                },
                PollCreateToken::Error(err) => {
                    println!("{}", err);
                    break;
                },
            }
        }
    }

    #[ignore = "not in ci"]
    #[tokio::test]
    async fn test_load() {
        let secret_store = SecretStore::load().await.unwrap();
        let token = BuilderIdToken::load(&secret_store).await;
        println!("{:?}", token);
        // println!("{:?}", token.unwrap().unwrap().access_token.0);
    }

    #[ignore = "not in ci"]
    #[tokio::test]
    async fn test_refresh() {
        let region = Region::new("us-east-1");
        let client = client(region.clone());
        let secret_store = SecretStore::load().await.unwrap();
        let token = BuilderIdToken::load(&secret_store).await.unwrap().unwrap();
        let token = token.refresh_token(&client, &secret_store, &region).await;
        println!("{:?}", token);
        // println!("{:?}", token.unwrap().unwrap().access_token.0);
    }
}
