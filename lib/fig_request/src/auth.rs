use std::fmt::Display;
use std::fs::{
    self,
    File,
};
use std::path::PathBuf;
use std::time::Duration;

use base64::prelude::*;
use fig_util::directories;
use jwt::{
    Header,
    RegisteredClaims,
    Token,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use thiserror::Error;

use crate::{
    Error,
    Request,
};

type Result<T, E = Error> = std::result::Result<T, E>;

async fn get_file_token() -> Result<String> {
    let mut creds = Credentials::load_credentials()?;
    if creds.is_expired() {
        creds.refresh_credentials().await?;
        creds.save_credentials()?;
    }

    creds.encode()
}

/// Gets the auth token from the environment or from the credentials file
pub async fn get_token() -> Result<String> {
    if let Ok(token) = std::env::var("FIG_TOKEN") {
        return Ok(token);
    }

    get_file_token().await
}

static EMAIL: once_cell::sync::OnceCell<Option<String>> = once_cell::sync::OnceCell::new();

/// Tries to get the email from the credentials file, if that fails, it will make a request to the
/// server to get the email and cache it in memory.
pub async fn get_email() -> Option<String> {
    if let Some(email) = EMAIL.get() {
        return email.clone();
    }

    if let Some(email) = Credentials::load_credentials().ok().and_then(|creds| creds.email) {
        return Some(email);
    }

    let email = if let Ok(val) = Request::get("/user/account").auth().json().await {
        val.get("email").and_then(|v| v.as_str()).map(|s| s.to_owned())
    } else {
        None
    };

    EMAIL.get_or_init(|| email).clone()
}

/// Prefer using [get_email] instead, this should only be used in cases where you can't await
pub fn get_email_sync() -> Option<String> {
    if let Some(email) = EMAIL.get() {
        return email.clone();
    }

    Credentials::load_credentials().ok().and_then(|creds| creds.email)
}

/// Checks if the user is logged in by checking if the FIG_TOKEN environment variable is set or if
/// the credentials file exists, this does not check if the credentials are valid
pub fn is_logged_in() -> bool {
    std::env::var("FIG_TOKEN").is_ok() || Credentials::load_credentials().is_ok()
}

pub fn logout() -> Result<()> {
    fig_settings::state::create_anonymous_id()?;
    fig_settings::state::remove_value("user_id").ok();
    fig_settings::state::remove_value("previous_email").ok();

    let creds = Credentials::default();
    creds.save_credentials()?;

    #[cfg(target_os = "macos")]
    {
        // This is old code and should probably be removed
        std::process::Command::new("defaults")
            .args(["delete", "com.amazon.codewhisperer.shared"])
            .output()
            .ok();
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignInInput {
    email: String,
}

#[derive(Debug, Error)]
pub enum SignInError {
    #[error(transparent)]
    Request(#[from] Error),
}

impl SignInInput {
    pub fn new(email: impl Into<String>) -> Self {
        Self { email: email.into() }
    }

    pub async fn sign_in(&self) -> Result<SignInOutput, Error> {
        Request::post("/auth/login/init")
            .body_json(&json!(self))
            .deser_json()
            .await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignInOutput {
    email: String,
    login_session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ConfirmOutput {
    #[serde(rename_all = "camelCase")]
    Success {
        email: String,
        access_token: String,
        id_token: String,
        refresh_token: String,
    },
    InvalidCode,
    TooManyAttempts,
}

#[derive(Debug, Error)]
pub enum SignInConfirmError {
    #[error("invalid code")]
    InvalidCode,
    #[error("too many attempts")]
    TooManyAttempts,
    #[error(transparent)]
    Request(#[from] Error),
}

impl SignInOutput {
    pub async fn confirm(&mut self, code: impl Into<String>) -> Result<Credentials, SignInConfirmError> {
        let resp = Request::post("/auth/login/confirm")
            .body_json(&json!({
                "email": self.email,
                "loginSessionId": self.login_session_id,
                "code": code.into(),
            }))
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await.map_err(Error::Reqwest)?;

        match serde_json::from_str::<ConfirmOutput>(&text) {
            Ok(ConfirmOutput::Success {
                email,
                access_token,
                id_token,
                refresh_token,
            }) =>
            {
                #[allow(deprecated)]
                Ok(Credentials {
                    email: Some(email),
                    credentials_type: CredentialsType::Jwt {
                        access_token: Some(access_token),
                        id_token: Some(id_token),
                        refresh_token: Some(refresh_token),
                        refresh_token_expired: Some(false),
                        expiration_time: None,
                    },
                })
            },
            Ok(ConfirmOutput::InvalidCode) => Err(SignInConfirmError::InvalidCode),
            Ok(ConfirmOutput::TooManyAttempts) => Err(SignInConfirmError::TooManyAttempts),
            Err(_) => Err(SignInConfirmError::Request(crate::parse_fig_error_response(
                status, text,
            ))),
        }
    }
}

fn rfc3339_deserialize_ignore_error<'de, D>(d: D) -> Result<Option<time::OffsetDateTime>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(time::serde::rfc3339::option::deserialize(d).ok().flatten())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum CredentialsType {
    Jwt {
        access_token: Option<String>,
        id_token: Option<String>,
        refresh_token: Option<String>,
        refresh_token_expired: Option<bool>,
        /// Expiration time was used in past, we keep it here for backward compatibility,
        /// in the future the JWT will be used to determine expiration time.
        #[deprecated]
        #[serde(
            serialize_with = "time::serde::rfc3339::option::serialize",
            deserialize_with = "rfc3339_deserialize_ignore_error"
        )]
        expiration_time: Option<time::OffsetDateTime>,
    },
    FigToken {
        fig_token: Option<String>,
    },
}

impl Default for CredentialsType {
    fn default() -> Self {
        Self::Jwt {
            access_token: None,
            id_token: None,
            refresh_token: None,
            refresh_token_expired: None,
            expiration_time: None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Credentials {
    pub email: Option<String>,
    #[serde(flatten)]
    pub credentials_type: CredentialsType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum RefreshResponse {
    #[serde(rename_all = "camelCase")]
    Success {
        access_token: Option<String>,
        id_token: Option<String>,
        refresh_token: Option<String>,
    },
    MissingRefreshToken,
    RefreshTokenExpired,
}

#[derive(Debug, Error)]
pub enum RefreshError {
    #[error("refresh token expired")]
    RefreshTokenExpired,
    #[error("refresh token not set")]
    RefreshTokenNotSet,
    #[error("cannot refresh fig token")]
    CannotRefreshFigToken,
}

impl Credentials {
    /// Path to the main credentials file
    pub fn path() -> Result<PathBuf, fig_util::directories::DirectoryError> {
        fig_util::directories::credentials_path()
    }

    /// Path to alternative credentials file folder
    pub fn account_credentials_dir() -> Result<PathBuf, fig_util::directories::DirectoryError> {
        Ok(directories::fig_data_dir()?.join("account_credentials"))
    }

    /// Path to credentials file for a specific account
    pub fn account_credentials_path(email: impl Display) -> Result<PathBuf, fig_util::directories::DirectoryError> {
        Ok(Credentials::account_credentials_dir()?.join(format!("{email}.json")))
    }

    pub fn new_jwt(
        email: impl Into<Option<String>>,
        access_token: Option<String>,
        id_token: Option<String>,
        refresh_token: Option<String>,
        refresh_token_expired: bool,
    ) -> Self {
        #[allow(deprecated)]
        Self {
            email: email.into(),
            credentials_type: CredentialsType::Jwt {
                access_token,
                id_token,
                refresh_token,
                refresh_token_expired: Some(refresh_token_expired),
                expiration_time: None,
            },
        }
    }

    pub fn new_fig_token(email: Option<String>, fig_token: Option<String>) -> Self {
        Self {
            email,
            credentials_type: CredentialsType::FigToken { fig_token },
        }
    }

    pub fn save_credentials(&self) -> Result<()> {
        let data_dir = directories::fig_data_dir()?;

        if !data_dir.exists() {
            fs::create_dir_all(&data_dir)?;
        }

        let mut creds_file = File::create(Credentials::path()?)?;

        #[cfg(unix)]
        {
            // Set permissions to 0600
            creds_file.set_permissions(std::os::unix::fs::PermissionsExt::from_mode(0o600))?;
        }

        serde_json::to_writer(&mut creds_file, self)?;

        Ok(())
    }

    pub fn load_credentials() -> Result<Credentials> {
        let creds_path = Credentials::path()?;

        if !creds_path.exists() {
            return Err(Error::NoToken);
        }

        Ok(serde_json::from_reader(File::open(creds_path)?)?)
    }

    pub async fn refresh_credentials(&mut self) -> Result<(), Error> {
        match &mut self.credentials_type {
            CredentialsType::Jwt {
                ref mut access_token,
                ref mut id_token,
                ref mut refresh_token,
                ref mut refresh_token_expired,
                ..
            } => {
                if let Ok(data_dir) = directories::fig_data_dir() {
                    if let Ok(token) = std::fs::read_to_string(data_dir.join("token_is_expired")) {
                        if refresh_token.as_deref().unwrap_or_default() == token {
                            return Err(RefreshError::RefreshTokenExpired.into());
                        }
                    }
                }

                if let Some(true) = refresh_token_expired {
                    return Err(RefreshError::RefreshTokenExpired.into());
                }

                let refresh_token_str = refresh_token.as_ref().ok_or(RefreshError::RefreshTokenNotSet)?;

                let resp = Request::post("/auth/refresh")
                    .body_json(&json!({
                        "refreshToken": refresh_token_str,
                    }))
                    .send()
                    .await?;

                let status = resp.status();
                let text = resp.text().await.map_err(Error::Reqwest)?;

                match serde_json::from_str::<RefreshResponse>(&text) {
                    Ok(RefreshResponse::Success {
                        access_token: new_access_token,
                        id_token: new_id_token,
                        refresh_token: new_refresh_token,
                    }) => {
                        if let Ok(data_dir) = directories::fig_data_dir() {
                            std::fs::remove_file(data_dir.join("token_is_expired")).ok();
                        }

                        *refresh_token = new_refresh_token;
                        *access_token = new_access_token;
                        *id_token = new_id_token;
                        *refresh_token_expired = Some(false);

                        Ok(())
                    },
                    Ok(RefreshResponse::MissingRefreshToken) => Err(RefreshError::RefreshTokenNotSet.into()),
                    Ok(RefreshResponse::RefreshTokenExpired) => {
                        if let Ok(data_dir) = directories::fig_data_dir() {
                            std::fs::write(
                                data_dir.join("token_is_expired"),
                                refresh_token.as_deref().unwrap_or_default(),
                            )
                            .ok();
                        }

                        *refresh_token_expired = Some(true);
                        self.save_credentials().ok();

                        Err(RefreshError::RefreshTokenExpired.into())
                    },
                    Err(_) => Err(crate::parse_fig_error_response(status, text)),
                }
            },
            CredentialsType::FigToken { .. } => Err(RefreshError::CannotRefreshFigToken.into()),
        }
    }

    /// Clear the values of the credentials
    pub fn clear_credentials(&mut self) {
        *self = Self::default();
    }

    pub fn is_expired_epsilon(&self, epsilon: Duration) -> bool {
        match &self.credentials_type {
            CredentialsType::Jwt { access_token, .. } => {
                let Some(access_token) = access_token else {
                    return true;
                };
                let Ok(token) = Token::<Header, RegisteredClaims, _>::parse_unverified(access_token) else {
                    return true;
                };
                let Some(expiration_time) = token.claims().expiration else {
                    return true;
                };
                let expiration_time = std::time::UNIX_EPOCH + Duration::from_secs(expiration_time);

                expiration_time + epsilon < std::time::SystemTime::now()
            },
            // Currently we are not tracking the expiration of FigTokens
            CredentialsType::FigToken { .. } => false,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.is_expired_epsilon(Duration::from_secs(30))
    }

    pub fn get_email(&self) -> Option<&String> {
        self.email.as_ref()
    }

    /// Encodes the credentials encoded for the Bearer header
    pub fn encode(&self) -> Result<String> {
        match &self.credentials_type {
            CredentialsType::Jwt {
                access_token, id_token, ..
            } => {
                if access_token.is_none() || id_token.is_none() {
                    return Err(Error::NoToken);
                }

                Ok(BASE64_STANDARD.encode(
                    json!({
                        "accessToken": access_token,
                        "idToken": id_token
                    })
                    .to_string(),
                ))
            },
            CredentialsType::FigToken { fig_token } => match fig_token {
                Some(fig_token) => Ok(fig_token.clone()),
                None => Err(Error::NoToken),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_credentials() -> Credentials {
        Credentials::new_jwt(
            "test@fig.io".to_owned(),
            Some("access_token".to_string()),
            None,
            None,
            false,
        )
    }

    #[test]
    fn save_load_credentials() {
        let original = Credentials::load_credentials();
        let mock = mock_credentials();
        mock.save_credentials().unwrap();
        let from_disk = Credentials::load_credentials().unwrap();
        if let Ok(original) = original {
            original.save_credentials().unwrap()
        }
        assert_eq!(mock, from_disk);
    }
}
