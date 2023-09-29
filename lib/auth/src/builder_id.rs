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
use aws_sdk_ssooidc::config::SharedAsyncSleep;
use aws_sdk_ssooidc::error::SdkError;
use aws_sdk_ssooidc::operation::create_token::CreateTokenOutput;
use aws_smithy_async::rt::sleep::TokioSleep;
use aws_types::app_name::AppName;
use aws_types::region::Region;
use once_cell::sync::Lazy;
use time::OffsetDateTime;
use tracing::error;

use crate::secret_store::SecretStore;
use crate::{
    Error,
    Result,
};

const CLIENT_NAME: &str = "CodeWhisperer for Terminal";

const OIDC_BUILDER_ID_ENDPOINT: &str = "https://oidc.us-east-1.amazonaws.com";
const OIDC_BUILDER_ID_REGION: Region = Region::from_static("us-east-1");

const SCOPES: &[&str] = &[
    "sso:account:access",
    "codewhisperer:completions",
    "codewhisperer:analysis",
];
const CLIENT_TYPE: &str = "public";
const START_URL: &str = "https://view.awsapps.com/start";
const GRANT_TYPE: &str = "urn:ietf:params:oauth:grant-type:device_code";

static APP_NAME: Lazy<AppName> =
    Lazy::new(|| AppName::new(format!("codewhisperer-terminal-v{}", env!("CARGO_PKG_VERSION"))).unwrap());

/// Indicates if an expiration time has passed, there is a small 1 min window that is removed
/// so the token will not expire in transit
fn is_expired(expiration_time: &OffsetDateTime) -> bool {
    let now = time::OffsetDateTime::now_utc();
    &(now + time::Duration::minutes(1)) > expiration_time
}

static CLIENT: Lazy<Client> = Lazy::new(|| {
    let retry_config = RetryConfig::standard().with_max_attempts(3);
    let sdk_config = aws_types::SdkConfig::builder()
        .endpoint_url(OIDC_BUILDER_ID_ENDPOINT)
        .region(OIDC_BUILDER_ID_REGION)
        .retry_config(retry_config)
        .sleep_impl(SharedAsyncSleep::new(TokioSleep::new()))
        .app_name(APP_NAME.clone())
        .build();
    Client::new(&sdk_config)
});

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeviceRegistration {
    pub client_id: String,
    pub client_secret: String,
    pub client_secret_expires_at: Option<time::OffsetDateTime>,
}

impl DeviceRegistration {
    const SECRET_KEY: &str = "codewhisper:builder-id:device-registration";

    fn load_from_secret_store(secret_store: &SecretStore) -> Result<Option<Self>> {
        let device_registration = secret_store.get(Self::SECRET_KEY)?;

        if let Some(device_registration) = device_registration {
            // check that the data is not expired, assume it is invalid if not present
            let device_registration: Self = serde_json::from_str(&device_registration)?;

            if let Some(client_secret_expires_at) = device_registration.client_secret_expires_at {
                if !is_expired(&client_secret_expires_at) {
                    return Ok(Some(device_registration));
                }
            }
        }

        Ok(None)
    }

    /// Register the client with OIDC and cache the response for the expiration period
    pub async fn register(secret_store: &SecretStore) -> Result<Self> {
        match Self::load_from_secret_store(secret_store) {
            Ok(Some(device_registration)) => return Ok(device_registration),
            Ok(None) => {},
            Err(err) => {
                error!(?err, "Failed to read device registration from keychain");
            },
        };

        let mut register = CLIENT
            .register_client()
            .client_name(CLIENT_NAME)
            .client_type(CLIENT_TYPE);
        for scope in SCOPES {
            register = register.scopes(*scope);
        }
        let register_res = register.send().await?;

        let res = Self {
            client_id: register_res.client_id.unwrap_or_default(),
            client_secret: register_res.client_secret.unwrap_or_default(),
            client_secret_expires_at: time::OffsetDateTime::from_unix_timestamp(register_res.client_secret_expires_at)
                .ok(),
        };

        if let Err(err) = secret_store.set(Self::SECRET_KEY, &serde_json::to_string(&res)?) {
            error!(?err, "Failed to write device registration to keychain");
        }

        Ok(res)
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
}

/// Init a builder id request
pub async fn start_device_authorization(secret_store: &SecretStore) -> Result<StartDeviceAuthorizationResponse> {
    let DeviceRegistration {
        client_id,
        client_secret,
        ..
    } = DeviceRegistration::register(secret_store).await?;

    let output = CLIENT
        .start_device_authorization()
        .client_id(&client_id)
        .client_secret(&client_secret)
        .start_url(START_URL)
        .send()
        .await?;

    Ok(StartDeviceAuthorizationResponse {
        device_code: output.device_code.unwrap_or_default(),
        user_code: output.user_code.unwrap_or_default(),
        verification_uri: output.verification_uri.unwrap_or_default(),
        verification_uri_complete: output.verification_uri_complete.unwrap_or_default(),
        expires_in: output.expires_in,
        interval: output.interval,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BuilderIdToken {
    pub access_token: String,
    pub expires_at: time::OffsetDateTime,
    pub refresh_token: Option<String>,
}

impl BuilderIdToken {
    const SECRET_KEY: &str = "codewhisper:builder-id:token";

    /// Load the token from the keychain, refresh the token if it is expired and return it
    pub async fn load(secret_store: &SecretStore) -> Result<Option<Self>> {
        match secret_store.get(Self::SECRET_KEY) {
            Ok(Some(access_token)) => {
                let token: Option<BuilderIdToken> = serde_json::from_str(&access_token)?;
                match token {
                    Some(token) => {
                        // if token is expired try to refresh
                        if token.is_expired() {
                            Ok(Some(refresh_token(secret_store).await?))
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

    /// If the time has passed the `expires_at` time
    ///
    /// The token is marked as expired 1 min before it actually does to account for the potential a
    /// token expires while in transit
    pub fn is_expired(&self) -> bool {
        is_expired(&self.expires_at)
    }

    /// Save the token to the keychain
    pub fn save(&self, secret_store: &SecretStore) -> Result<()> {
        secret_store.set(Self::SECRET_KEY, &serde_json::to_string(self)?)?;
        Ok(())
    }

    /// Delete the token from the keychain
    pub fn delete(&self, secret_store: &SecretStore) -> Result<()> {
        secret_store.delete(Self::SECRET_KEY)?;
        Ok(())
    }
}

impl From<CreateTokenOutput> for BuilderIdToken {
    fn from(output: CreateTokenOutput) -> Self {
        Self {
            access_token: output.access_token.unwrap_or_default(),
            expires_at: time::OffsetDateTime::now_utc() + time::Duration::seconds(output.expires_in as i64),
            refresh_token: output.refresh_token,
        }
    }
}

pub enum PollCreateToken {
    Pending,
    Complete(BuilderIdToken),
    Error(Error),
}

/// Poll for the create token response
pub async fn poll_create_token(device_code: String, secret_store: &SecretStore) -> PollCreateToken {
    let DeviceRegistration {
        client_id,
        client_secret,
        ..
    } = match DeviceRegistration::register(secret_store).await {
        Ok(res) => res,
        Err(err) => {
            return PollCreateToken::Error(err);
        },
    };

    match CLIENT
        .create_token()
        .grant_type(GRANT_TYPE)
        .device_code(device_code)
        .client_id(client_id)
        .client_secret(client_secret)
        .send()
        .await
    {
        Ok(output) => {
            let token: BuilderIdToken = output.into();

            if let Err(err) = token.save(secret_store) {
                error!(?err, "Failed to store builder id token");
            };

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

/// Refresh the access token
pub async fn refresh_token(secret_store: &SecretStore) -> Result<BuilderIdToken> {
    let DeviceRegistration {
        client_id,
        client_secret,
        ..
    } = DeviceRegistration::register(secret_store).await?;

    match CLIENT
        .create_token()
        .client_id(client_id)
        .client_secret(client_secret)
        .send()
        .await
    {
        Ok(output) => {
            let token: BuilderIdToken = output.into();

            if let Err(err) = token.save(secret_store) {
                error!(?err, "Failed to store builder id access token");
            };

            Ok(token)
        },
        Err(err) => {
            error!(?err, "Failed to refresh builder id access token");
            Err(err.into())
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_name() {
        println!("{:?}", *APP_NAME);
    }

    #[test]
    fn client() {
        println!("{:?}", *CLIENT);
    }
}
