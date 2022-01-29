use aws_sdk_cognitoidentityprovider::{
    error::{
        InitiateAuthError, InitiateAuthErrorKind, RespondToAuthChallengeError,
        RespondToAuthChallengeErrorKind, SignUpErrorKind, UpdateUserAttributesError,
        UserLambdaValidationException,
    },
    model::{AttributeType, AuthFlowType, ChallengeNameType},
    AppName, Client, Config, Region, SdkError,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::HashMap,
    fs::{self, File},
};
use thiserror::Error;

pub fn get_client(client_name: impl Into<Cow<'static, str>>) -> anyhow::Result<Client> {
    let config = Config::builder()
        .app_name(AppName::new(client_name)?)
        .region(Region::new("us-east-1"))
        .build();

    Ok(Client::from_conf(config))
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationErrorDetail {
    message: String,
    path: Vec<String>,
    r#type: String,
}

#[derive(Debug, Error)]
#[error("Validation error: {:?}", details.get(0).map(|d| &d.message))]
pub struct ValidationError {
    details: Vec<ValidationErrorDetail>,
}

fn parse_lambda_error(error: UserLambdaValidationException) -> anyhow::Result<ValidationError> {
    let lambda_triggers = [
        "PreSignUp",
        "PostConfirmation",
        "PreAuthentication",
        "DefineAuthChallenge",
        "CreateAuthChallenge",
        "VerifyAuthChallengeResponse",
        "PreTokenGeneration",
        "PostAuthentication",
    ];

    if let Some(ref message) = error.message {
        for lambda_trigger in lambda_triggers {
            let lambda_trigger_prefix = format!("{} failed with error ", lambda_trigger);
            if !message.starts_with(&lambda_trigger_prefix) {
                continue;
            }

            let message = &message[lambda_trigger_prefix.len()..];

            match message.strip_prefix("ValidationError=") {
                Some(message) => {
                    let message = message.strip_prefix("ValidationError=").unwrap();
                    let message = &message["ValidationError=".len()..];

                    let details = match serde_json::from_str::<Vec<ValidationErrorDetail>>(message)
                    {
                        Ok(details) => details,
                        Err(err) => {
                            return Err(err.into());
                        }
                    };

                    return Ok(ValidationError { details });
                }
                None => return Err(error.into()),
            }
        }
    }

    Err(error.into())
}

pub struct SignInInput<'a> {
    client: &'a Client,
    client_id: String,
    username_or_email: String,
}

#[derive(Debug, Error)]
pub enum SignInError {
    #[error("user not found: {0:?}")]
    UserNotFound(Option<String>),
    #[error("missing session")]
    MissingSession,
    #[error("missing challenge name")]
    MissingChallengeName,
    #[error("missing challenge parameters")]
    MissingChallengeParameters,
    #[error("sdk error")]
    SdkError(#[from] Box<SdkError<InitiateAuthError>>),
}

impl<'a> SignInInput<'a> {
    pub fn new(
        client: &'a Client,
        client_id: impl Into<String>,
        username_or_email: impl Into<String>,
    ) -> Self {
        Self {
            client,
            client_id: client_id.into(),
            username_or_email: username_or_email.into(),
        }
    }

    pub async fn sign_in(&self) -> Result<SignInOutput<'a>, SignInError> {
        let auth_result = self
            .client
            .initiate_auth()
            .client_id(&self.client_id)
            .auth_flow(AuthFlowType::CustomAuth)
            .auth_parameters("USERNAME", &self.username_or_email)
            .client_metadata("CUSTOM_AUTH", "PASSWORDLESS_EMAIL")
            .send()
            .await;

        let output = auth_result.map_err(|err| match err {
            SdkError::ServiceError {
                err: ref auth_err, ..
            } => match auth_err.kind {
                InitiateAuthErrorKind::UserNotFoundException(ref user_not_found) => {
                    SignInError::UserNotFound(user_not_found.message.clone())
                }
                _ => SignInError::SdkError(Box::new(err)),
            },
            err => SignInError::SdkError(Box::new(err)),
        })?;

        let session = output.session.ok_or(SignInError::MissingSession)?;

        let challenge_name = output
            .challenge_name
            .ok_or(SignInError::MissingChallengeName)?;

        let challenge_parameters = output
            .challenge_parameters
            .ok_or(SignInError::MissingChallengeParameters)?;

        Ok(SignInOutput {
            client: self.client,
            client_id: self.client_id.clone(),
            username_or_email: self.username_or_email.clone(),
            session,
            challenge_name,
            challenge_parameters,
        })
    }
}

pub struct SignInOutput<'a> {
    client: &'a Client,
    client_id: String,
    username_or_email: String,
    session: String,
    challenge_name: ChallengeNameType,
    challenge_parameters: HashMap<String, String>,
}

#[derive(Debug, Error)]
pub enum SignInConfirmError {
    #[error("error code mismatch")]
    ErrorCodeMismatch,
    #[error("not authorized")]
    NotAuthorized,
    #[error("could not sign in")]
    CouldNotSignIn,
    #[error("validation error")]
    ValidationError(#[from] ValidationError),
    #[error("sdk error")]
    SdkError(#[from] Box<SdkError<RespondToAuthChallengeError>>),
}

impl<'a> SignInOutput<'a> {
    pub async fn confirm(
        &mut self,
        code: impl Into<String>,
    ) -> Result<Credentials, SignInConfirmError> {
        let respond_to_auth_result = self
            .client
            .respond_to_auth_challenge()
            .client_id(&self.client_id)
            .session(&self.session)
            .challenge_name(self.challenge_name.clone())
            .challenge_responses("USERNAME", &self.username_or_email)
            .challenge_responses("ANSWER", code)
            .send()
            .await;

        let out = respond_to_auth_result.map_err(|err| match err {
            SdkError::ServiceError {
                err: ref auth_err, ..
            } => match auth_err.kind {
                RespondToAuthChallengeErrorKind::UserLambdaValidationException(ref error) => {
                    match parse_lambda_error(error.clone()) {
                        Ok(err) => SignInConfirmError::ValidationError(err),
                        _ => SignInConfirmError::SdkError(Box::new(err)),
                    }
                }
                RespondToAuthChallengeErrorKind::NotAuthorizedException(_) => {
                    SignInConfirmError::NotAuthorized
                }
                _ => SignInConfirmError::SdkError(Box::new(err)),
            },
            err => SignInConfirmError::SdkError(Box::new(err)),
        })?;

        match out.authentication_result {
            Some(auth_result) => Ok(Credentials::new(
                self.username_or_email.clone(),
                auth_result.access_token,
                auth_result.id_token,
                auth_result.refresh_token,
                auth_result.expires_in,
            )),
            None => match out.session {
                Some(session) => {
                    self.session = session;
                    if let Some(challenge_name) = out.challenge_name {
                        self.challenge_name = challenge_name;
                    }
                    if let Some(challenge_parameters) = out.challenge_parameters {
                        self.challenge_parameters = challenge_parameters;
                    }
                    Err(SignInConfirmError::ErrorCodeMismatch)
                }
                None => Err(SignInConfirmError::CouldNotSignIn),
            },
        }
    }
}

pub struct SignUpInput<'a> {
    client: &'a Client,
    client_id: String,
    email: String,
}

#[derive(Debug, Error)]
pub enum SignUpError {
    #[error("sdk error")]
    SdkError(#[from] Box<SdkError<aws_sdk_cognitoidentityprovider::error::SignUpError>>),
    #[error("validation error")]
    ValidationError(#[from] ValidationError),
}

impl<'a> SignUpInput<'a> {
    pub fn new(client: &'a Client, client_id: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            client,
            client_id: client_id.into(),
            email: email.into(),
        }
    }

    pub async fn sign_up(self) -> Result<(), SignUpError> {
        let password = generate_password(32);
        let username = uuid::Uuid::new_v4().to_hyphenated().to_string();

        let sign_up_result = self
            .client
            .sign_up()
            .client_id(&self.client_id)
            .username(&username)
            .password(&password)
            .user_attributes(
                AttributeType::builder()
                    .name("email")
                    .value(&self.email)
                    .build(),
            )
            .send()
            .await;

        sign_up_result.map_err(|err| match err {
            SdkError::ServiceError {
                err: ref sign_up_err,
                ..
            } => match sign_up_err.kind {
                SignUpErrorKind::UserLambdaValidationException(ref error) => {
                    match parse_lambda_error(error.clone()) {
                        Ok(err) => SignUpError::ValidationError(err),
                        _ => SignUpError::SdkError(Box::new(err)),
                    }
                }
                _ => SignUpError::SdkError(Box::new(err)),
            },
            err => SignUpError::SdkError(Box::new(err)),
        })?;

        Ok(())
    }
}

pub struct ChangeUsernameInput {
    client: Client,
    username: String,
    access_token: String,
}

impl ChangeUsernameInput {
    pub fn new(
        client: Client,
        username: impl Into<String>,
        access_token: impl Into<String>,
    ) -> Self {
        Self {
            client,
            username: username.into(),
            access_token: access_token.into(),
        }
    }

    pub async fn change_username(self) -> Result<(), SdkError<UpdateUserAttributesError>> {
        self.client
            .update_user_attributes()
            .access_token(&self.access_token)
            .user_attributes(
                AttributeType::builder()
                    .name("preferred_username")
                    .value(&self.username)
                    .build(),
            )
            .send()
            .await?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials {
    pub email: Option<String>,
    pub access_token: Option<String>,
    pub id_token: Option<String>,
    pub refresh_token: Option<String>,
    pub experation_time: time::OffsetDateTime,
}

impl Credentials {
    pub fn new(
        email: impl Into<String>,
        access_token: Option<String>,
        id_token: Option<String>,
        refresh_token: Option<String>,
        expires_in: i32,
    ) -> Self {
        Self {
            email: Some(email.into()),
            access_token,
            id_token,
            refresh_token,
            experation_time: time::OffsetDateTime::now_utc()
                + time::Duration::seconds(expires_in.into()),
        }
    }

    pub fn save_credentials(&self) -> anyhow::Result<()> {
        let cache =
            dirs::cache_dir().ok_or_else(|| anyhow::anyhow!("Could not find cache directory"))?;

        let fig_cache = cache.join("fig");

        if !fig_cache.exists() {
            fs::create_dir_all(&fig_cache)?;
        }

        let mut file = File::create(fig_cache.join("credentials.json"))?;

        #[cfg(unix)]
        {
            // Set permissions to 0600
            file.set_permissions(std::os::unix::fs::PermissionsExt::from_mode(0o600))?;
        }

        serde_json::to_writer(&mut file, self)?;

        Ok(())
    }

    pub fn load_credentials() -> anyhow::Result<Credentials> {
        let cache =
            dirs::cache_dir().ok_or_else(|| anyhow::anyhow!("Could not find cache directory"))?;

        let fig_cache = cache.join("fig");

        if !fig_cache.exists() {
            fs::create_dir_all(&fig_cache)?;
        }

        let file = File::open(fig_cache.join("credentials.json"))?;

        Ok(serde_json::from_reader(file)?)
    }

    pub async fn refresh_credentials(
        &mut self,
        client: &Client,
        client_id: &str,
    ) -> anyhow::Result<()> {
        let refresh_token = self
            .refresh_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Refresh token is not set"))?;

        let out = client
            .initiate_auth()
            .client_id(client_id)
            .auth_flow(AuthFlowType::RefreshTokenAuth)
            .auth_parameters("REFRESH_TOKEN", refresh_token)
            .send()
            .await?;

        match out.authentication_result {
            Some(auth_result) => {
                self.access_token = auth_result.access_token;
                self.id_token = auth_result.id_token;
                self.experation_time = time::OffsetDateTime::now_utc()
                    + time::Duration::seconds(auth_result.expires_in.into());
            }
            None => return Err(anyhow::anyhow!("Could not refresh credentials")),
        }

        Ok(())
    }

    /// Clear the values of the credentials
    pub fn clear_cridentials(&mut self) {
        self.email = None;
        self.access_token = None;
        self.id_token = None;
        self.refresh_token = None;
        self.experation_time = time::OffsetDateTime::UNIX_EPOCH;
    }

    pub fn get_access_token(&self) -> Option<&String> {
        self.access_token.as_ref()
    }

    pub fn get_id_token(&self) -> Option<&String> {
        self.id_token.as_ref()
    }

    pub fn get_refresh_token(&self) -> Option<&String> {
        self.refresh_token.as_ref()
    }

    pub fn get_experation_time(&self) -> &time::OffsetDateTime {
        &self.experation_time
    }

    pub fn get_email(&self) -> Option<&String> {
        self.email.as_ref()
    }
}

/// Generates a password of the given length
///
/// The password is guaranteed to include at least one lowercase letter,
/// one uppercase letter, one digit, and one special character.
///
/// Length must be greater than or equal to 4
fn generate_password(length: usize) -> String {
    assert!(length >= 4);

    let special = r#"^$*.[]{}()?-"!@#%&/\,><':;|_~`+="#.chars().collect::<Vec<_>>();
    let alphanum = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
        .chars()
        .collect::<Vec<_>>();
    let all_characters = special.iter().chain(alphanum.iter()).collect::<Vec<_>>();

    let mut rng = rand::thread_rng();

    loop {
        let mut password = String::with_capacity(length);

        for _ in 0..length {
            password.push(*all_characters[rng.gen_range(0..all_characters.len())]);
        }

        // Check for lowercase, uppercase, digit, and special character
        if password.chars().any(|c| c.is_numeric())
            && password.chars().any(|c| special.contains(&c))
            && password.chars().any(|c| c.is_ascii_uppercase())
            && password.chars().any(|c| c.is_ascii_lowercase())
        {
            return password;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_password() {
        for _ in 0..64 {
            let password = generate_password(32);
            assert!(password.chars().any(|c| c.is_numeric()));
            assert!(password.chars().any(|c| c.is_ascii_uppercase()));
            assert!(password.chars().any(|c| c.is_ascii_lowercase()));
            assert!(password.chars().any(|c| c.is_ascii_punctuation()));
        }
    }
}
