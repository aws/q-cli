use std::fmt::Display;
use std::fs::{
    self,
    File,
};
use std::path::PathBuf;

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

use crate::Request;

type Result<T, E = crate::Error> = std::result::Result<T, E>;

pub async fn get_token() -> Result<String> {
    let mut creds = Credentials::load_credentials()?;
    if creds.is_expired() {
        creds.refresh_credentials().await?;
        creds.save_credentials()?;
    }

    match (creds.get_access_token(), creds.get_refresh_token()) {
        (None, _) => Err(crate::Error::NoToken),
        // TODO: Migrate those with only `access_token`
        (Some(_), None) => Ok(creds.encode()),
        (Some(_), Some(_)) => Ok(creds.encode()),
    }
}

pub fn get_email() -> Option<String> {
    Credentials::load_credentials().ok().and_then(|creds| creds.email)
}

pub fn is_logged_in() -> bool {
    get_email().is_some()
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
            .args(["delete", "com.mschrage.fig.shared"])
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
    Request(#[from] crate::Error),
}

impl SignInInput {
    pub fn new(email: impl Into<String>) -> Self {
        Self { email: email.into() }
    }

    pub async fn sign_in(&self) -> Result<SignInOutput, crate::Error> {
        Request::post("/auth/login/init").body(&json!(self)).deser_json().await
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
    Request(#[from] crate::Error),
}

impl SignInOutput {
    pub async fn confirm(&mut self, code: impl Into<String>) -> Result<Credentials, SignInConfirmError> {
        let resp = Request::post("/auth/login/confirm")
            .body(&json!({
                "email": self.email,
                "loginSessionId": self.login_session_id,
                "code": code.into(),
            }))
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await.map_err(crate::Error::Reqwest)?;

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
                    access_token: Some(access_token),
                    id_token: Some(id_token),
                    refresh_token: Some(refresh_token),
                    refresh_token_expired: Some(false),
                    expiration_time: None,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Credentials {
    pub email: Option<String>,
    pub access_token: Option<String>,
    pub id_token: Option<String>,
    pub refresh_token: Option<String>,
    pub refresh_token_expired: Option<bool>,
    /// Expiration time was used in past, we keep it here for backward compatibility,
    /// in the future the JWT will be used to determine expiration time.
    #[deprecated]
    #[serde(
        serialize_with = "time::serde::rfc3339::option::serialize",
        deserialize_with = "rfc3339_deserialize_ignore_error"
    )]
    expiration_time: Option<time::OffsetDateTime>,
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

    pub fn new(
        email: impl Into<String>,
        access_token: Option<String>,
        id_token: Option<String>,
        refresh_token: Option<String>,
        refresh_token_expired: bool,
    ) -> Self {
        #[allow(deprecated)]
        Self {
            email: Some(email.into()),
            access_token,
            id_token,
            refresh_token,
            refresh_token_expired: Some(refresh_token_expired),
            expiration_time: None,
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

        #[cfg(target_os = "macos")]
        {
            use crate::defaults::{
                remove_default,
                set_default,
            };

            match &self.id_token {
                Some(id) => {
                    if option_env!("FIG_MACOS_BACKPORT").is_none() {
                        set_default("id_token", id)?;
                    }
                },
                None => {
                    remove_default("id_token").ok();
                },
            }

            match &self.access_token {
                Some(access) => {
                    if option_env!("FIG_MACOS_BACKPORT").is_none() {
                        set_default("access_token", access)?;
                    }
                },
                None => {
                    remove_default("access_token").ok();
                },
            }

            match &self.refresh_token {
                Some(refresh) => {
                    if option_env!("FIG_MACOS_BACKPORT").is_none() {
                        set_default("refresh_token", refresh)?;
                    }
                },
                None => {
                    remove_default("refresh_token").ok();
                },
            }

            match &self.email {
                Some(email) => {
                    if option_env!("FIG_MACOS_BACKPORT").is_none() {
                        set_default("userEmail", email)?;
                    }
                },
                None => {
                    remove_default("userEmail").ok();
                },
            }
        }

        Ok(())
    }

    pub fn load_credentials() -> Result<Credentials> {
        let creds_path = Credentials::path()?;

        if !creds_path.exists() {
            return Err(crate::Error::NoToken);
        }

        Ok(serde_json::from_reader(File::open(creds_path)?)?)
    }

    pub async fn refresh_credentials(&mut self) -> Result<(), crate::Error> {
        if let Ok(data_dir) = directories::fig_data_dir() {
            if let Ok(token) = std::fs::read_to_string(data_dir.join("token_is_expired")) {
                if self.refresh_token.as_deref().unwrap_or_default() == token {
                    return Err(RefreshError::RefreshTokenExpired.into());
                }
            }
        }

        if let Some(true) = self.refresh_token_expired {
            return Err(RefreshError::RefreshTokenExpired.into());
        }

        let refresh_token = self.refresh_token.as_ref().ok_or(RefreshError::RefreshTokenNotSet)?;

        let resp = Request::post("/auth/refresh")
            .body(&json!({
                "refreshToken": refresh_token,
            }))
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await.map_err(crate::Error::Reqwest)?;

        match serde_json::from_str::<RefreshResponse>(&text) {
            Ok(RefreshResponse::Success {
                access_token,
                id_token,
                refresh_token,
            }) => {
                if let Ok(data_dir) = directories::fig_data_dir() {
                    std::fs::remove_file(data_dir.join("token_is_expired")).ok();
                }

                self.refresh_token = refresh_token;
                self.access_token = access_token;
                self.id_token = id_token;
                self.refresh_token_expired = Some(false);

                Ok(())
            },
            Ok(RefreshResponse::MissingRefreshToken) => Err(RefreshError::RefreshTokenNotSet.into()),
            Ok(RefreshResponse::RefreshTokenExpired) => {
                if let Ok(data_dir) = directories::fig_data_dir() {
                    std::fs::write(
                        data_dir.join("token_is_expired"),
                        self.refresh_token.as_deref().unwrap_or_default(),
                    )
                    .ok();
                }

                self.refresh_token_expired = Some(true);
                self.save_credentials().ok();

                Err(RefreshError::RefreshTokenExpired.into())
            },
            Err(_) => Err(crate::parse_fig_error_response(status, text)),
        }
    }

    /// Clear the values of the credentials
    pub fn clear_credentials(&mut self) {
        *self = Self::default();
    }

    pub fn get_access_token(&self) -> Option<&str> {
        self.access_token.as_deref()
    }

    pub fn get_id_token(&self) -> Option<&str> {
        self.id_token.as_deref()
    }

    pub fn get_refresh_token(&self) -> Option<&str> {
        self.refresh_token.as_deref()
    }

    pub fn get_expiration_time(&self) -> Option<time::OffsetDateTime> {
        let access_token = self.access_token.as_ref()?;
        let token: Token<Header, RegisteredClaims, _> = Token::parse_unverified(access_token).ok()?;
        time::OffsetDateTime::from_unix_timestamp(token.claims().expiration?.try_into().ok()?).ok()
    }

    pub fn is_expired_epsilon(&self, epsilon: time::Duration) -> bool {
        match self.get_expiration_time() {
            Some(expiration_time) => expiration_time + epsilon < time::OffsetDateTime::now_utc(),
            None => true,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.is_expired_epsilon(time::Duration::seconds(30))
    }

    pub fn get_email(&self) -> Option<&String> {
        self.email.as_ref()
    }

    /// Encodes the credentials as a base64 string for authentication
    pub fn encode(&self) -> String {
        base64::encode(
            json!({
                "accessToken": self.access_token,
                "idToken": self.id_token
            })
            .to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_credentials() -> Credentials {
        Credentials::new("test@fig.io", Some("access_token".to_string()), None, None, false)
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
