use aws_sdk_cognitoidentityprovider::{
    error::{
        ConfirmSignUpError, ConfirmSignUpErrorKind, InitiateAuthError, InitiateAuthErrorKind,
        ResendConfirmationCodeError, RespondToAuthChallengeError, RespondToAuthChallengeErrorKind,
        SignUpErrorKind, UpdateUserAttributesError, UserLambdaValidationException,
    },
    model::{AttributeType, AuthFlowType, ChallengeNameType},
    AppName, Client, Config, Region, SdkError,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap, fs::{File, self}};
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
    #[error("User not found: {:?}", _0)]
    UserNotFound(Option<String>),
    #[error("Missing Session")]
    MissingSession,
    #[error("Missing Challenge Name")]
    MissingChallengeName,
    #[error("Missing Challenge Parameters")]
    MissingChallengeParameters,
    #[error("Sdk Error: {:?}", _0)]
    SdkError(Box<SdkError<InitiateAuthError>>),
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

    pub async fn sign_in(self) -> Result<SignInOutput<'a>, SignInError> {
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
            client_id: self.client_id,
            username_or_email: self.username_or_email,
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
    #[error("Validation Error: {:?}", _0)]
    ValidationError(ValidationError),
    #[error("Error Code Mismatch")]
    ErrorCodeMismatch,
    #[error("Could not sign in")]
    CouldNotSignIn,
    #[error("Sdk Error: {:?}", _0)]
    SdkError(Box<SdkError<RespondToAuthChallengeError>>),
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
                _ => SignInConfirmError::SdkError(Box::new(err)),
            },
            err => SignInConfirmError::SdkError(Box::new(err)),
        })?;

        match out.authentication_result {
            Some(auth_result) => Ok(Credentials {
                access_token: auth_result.access_token,
                id_token: auth_result.id_token,
                refresh_token: auth_result.refresh_token,
                expires_in: auth_result.expires_in,
            }),
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
    #[error("Sdk Error: {:?}", _0)]
    SdkError(Box<SdkError<aws_sdk_cognitoidentityprovider::error::SignUpError>>),
    #[error("Validation Error: {:?}", _0)]
    ValidationError(ValidationError),
}

impl<'a> SignUpInput<'a> {
    pub fn new(client: &'a Client, client_id: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            client,
            client_id: client_id.into(),
            email: email.into(),
        }
    }

    pub async fn sign_up(self) -> Result<SignUpOutput<'a>, SignUpError> {
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

        let out = sign_up_result.map_err(|err| match err {
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

        Ok(SignUpOutput {
            client: self.client,
            client_id: self.client_id,
            username,
            password,
            user_sub: out.user_sub,
            user_confirmed: out.user_confirmed,
        })
    }
}

pub struct SignUpOutput<'a> {
    client: &'a Client,
    client_id: String,
    pub username: String,
    pub password: String,
    pub user_sub: Option<String>,
    pub user_confirmed: bool,
}

#[derive(Debug, Error)]
pub enum SignUpConfirmError {
    #[error("Email Exists: {:?}", _0)]
    EmailExists(Option<String>),
    #[error("Code Mismatch: {:?}", _0)]
    CodeMismatch(Option<String>),
    #[error("Expired Code: {:?}", _0)]
    ExpiredCode(Option<String>),
    #[error("Could not sign up")]
    CouldNotSignUp,
    #[error("Validation Error: {:?}", _0)]
    ValidationError(ValidationError),
    #[error("Sdk Error: {:?}", _0)]
    SdkErrorConfirmSignUp(SdkError<ConfirmSignUpError>),
    #[error("Sdk Error: {:?}", _0)]
    SdkErrorInitiateAuth(SdkError<InitiateAuthError>),
}

impl<'a> SignUpOutput<'a> {
    pub async fn confirm(
        &mut self,
        code: impl Into<String>,
    ) -> Result<Credentials, SignUpConfirmError> {
        let confirm_sign_up_result = self
            .client
            .confirm_sign_up()
            .client_id(&self.client_id)
            .username(&self.username)
            .confirmation_code(code)
            .send()
            .await;

        confirm_sign_up_result.map_err(|err| match err {
            SdkError::ServiceError {
                err: ref auth_err, ..
            } => match auth_err.kind {
                ConfirmSignUpErrorKind::AliasExistsException(ref error) => {
                    SignUpConfirmError::EmailExists(error.message.clone())
                }
                ConfirmSignUpErrorKind::CodeMismatchException(ref error) => {
                    SignUpConfirmError::CodeMismatch(error.message.clone())
                }
                ConfirmSignUpErrorKind::ExpiredCodeException(ref error) => {
                    SignUpConfirmError::ExpiredCode(error.message.clone())
                }
                ConfirmSignUpErrorKind::UserLambdaValidationException(ref error) => {
                    match parse_lambda_error(error.clone()) {
                        Ok(err) => SignUpConfirmError::ValidationError(err),
                        _ => SignUpConfirmError::SdkErrorConfirmSignUp(err),
                    }
                }
                _ => SignUpConfirmError::SdkErrorConfirmSignUp(err),
            },
            err => SignUpConfirmError::SdkErrorConfirmSignUp(err),
        })?;

        self.user_confirmed = true;

        let initiate_auth = self
            .client
            .initiate_auth()
            .client_id(&self.client_id)
            .auth_flow(AuthFlowType::UserPasswordAuth)
            .auth_parameters("USERNAME", &self.username)
            .auth_parameters("PASSWORD", &self.password)
            .send()
            .await;

        let out = initiate_auth.map_err(|err| match err {
            SdkError::ServiceError {
                err: ref auth_err, ..
            } => match auth_err.kind {
                InitiateAuthErrorKind::UserLambdaValidationException(ref error) => {
                    match parse_lambda_error(error.clone()) {
                        Ok(err) => SignUpConfirmError::ValidationError(err),
                        _ => SignUpConfirmError::SdkErrorInitiateAuth(err),
                    }
                }
                _ => SignUpConfirmError::SdkErrorInitiateAuth(err),
            },
            err => SignUpConfirmError::SdkErrorInitiateAuth(err),
        })?;

        match out.authentication_result {
            Some(auth_result) => Ok(Credentials {
                access_token: auth_result.access_token,
                id_token: auth_result.id_token,
                refresh_token: auth_result.refresh_token,
                expires_in: auth_result.expires_in,
            }),
            None => Err(SignUpConfirmError::CouldNotSignUp),
        }
    }

    pub async fn resend(&self) -> Result<(), SdkError<ResendConfirmationCodeError>> {
        self.client
            .resend_confirmation_code()
            .client_id(&self.client_id)
            .username(&self.username)
            .send()
            .await?;

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
    pub access_token: Option<String>,
    pub id_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_in: i32,
}

impl Credentials {
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

    loop {
        let mut rng = rand::thread_rng();
        let mut password = String::with_capacity(length);

        for _ in 0..length {
            password.push(*all_characters[rng.gen_range(0..all_characters.len())]);
        }

        // Check for number
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
