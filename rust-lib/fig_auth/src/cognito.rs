use std::collections::HashMap;
use std::fs::{
    self,
    File,
};

use aws_sdk_cognitoidentityprovider::error::{
    InitiateAuthError,
    InitiateAuthErrorKind,
    RespondToAuthChallengeError,
    RespondToAuthChallengeErrorKind,
    SignUpErrorKind,
    UpdateUserAttributesError,
    UserLambdaValidationException,
};
use aws_sdk_cognitoidentityprovider::model::{
    AttributeType,
    AuthFlowType,
    ChallengeNameType,
};
use aws_sdk_cognitoidentityprovider::types::SdkError;
use aws_sdk_cognitoidentityprovider::{
    AppName,
    Client,
    Config,
    Region,
    RetryConfig,
};
use aws_smithy_client::erase::{
    DynConnector,
    DynMiddleware,
};
use aws_smithy_client::hyper_ext;
use fig_util::directories;
use jwt::{
    Header,
    RegisteredClaims,
    Token,
};
use serde::{
    Deserialize,
    Deserializer,
    Serialize,
};
use serde_json::json;
use thiserror::Error;

use crate::password::generate_password;
use crate::{
    defaults,
    CLIENT_ID,
    REGION,
};

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Cognito(#[from] aws_sdk_cognitoidentityprovider::Error),
    #[error(transparent)]
    Refresh(#[from] RefreshError),
    #[error(transparent)]
    UserLambdaValidation(#[from] aws_sdk_cognitoidentityprovider::error::UserLambdaValidationException),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Defaults(#[from] defaults::DefaultsError),
    #[error("could not find dir")]
    Dir,
    #[error("credentials file does not exist")]
    CredentialsFileNotExist,
}

pub fn get_client() -> Result<aws_sdk_cognitoidentityprovider::Client> {
    let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_webpki_roots()
        .https_only()
        .enable_http1()
        .build();

    let hyper_connector = hyper_ext::Adapter::builder().build(https);

    let mut client: aws_smithy_client::Client<DynConnector, DynMiddleware<DynConnector>> =
        aws_smithy_client::Builder::new()
            .connector(DynConnector::new(hyper_connector))
            .middleware(DynMiddleware::new(
                aws_sdk_cognitoidentityprovider::middleware::DefaultMiddleware::new(),
            ))
            .build();

    client.set_sleep_impl(None);
    client.set_retry_config(RetryConfig::disabled().into());

    let config = Config::builder()
        .region(Region::new(REGION))
        .app_name(AppName::new("rust-client").unwrap())
        .build();

    Ok(aws_sdk_cognitoidentityprovider::Client::with_config(client, config))
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

fn parse_lambda_error(error: UserLambdaValidationException) -> Result<ValidationError> {
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

                    let details = match serde_json::from_str::<Vec<ValidationErrorDetail>>(message) {
                        Ok(details) => details,
                        Err(err) => {
                            return Err(err.into());
                        },
                    };

                    return Ok(ValidationError { details });
                },
                None => return Err(error.into()),
            }
        }
    }

    Err(error.into())
}

pub struct SignInInput<'a> {
    client: &'a Client,
    client_id: Option<String>,
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
    #[error(transparent)]
    SdkInitiateAuthError(#[from] Box<SdkError<InitiateAuthError>>),
    #[error(transparent)]
    SdkRespondToAuthChallengeError(#[from] Box<SdkError<RespondToAuthChallengeError>>),
}

impl<'a> SignInInput<'a> {
    pub fn new(client: &'a Client, username_or_email: impl Into<String>, client_id: impl Into<Option<String>>) -> Self {
        Self {
            client,
            client_id: client_id.into(),
            username_or_email: username_or_email.into(),
        }
    }

    pub async fn sign_in(&self) -> Result<SignInOutput<'a>, SignInError> {
        let client_id = self.client_id.as_ref().map_or(CLIENT_ID, String::as_str);

        let initiate_auth_result = self
            .client
            .initiate_auth()
            .client_id(client_id)
            .auth_flow(AuthFlowType::CustomAuth)
            .auth_parameters("USERNAME", &self.username_or_email)
            .send()
            .await;

        let initiate_auth_output = initiate_auth_result.map_err(|err| match err {
            SdkError::ServiceError { err: ref auth_err, .. } => match auth_err.kind {
                InitiateAuthErrorKind::UserNotFoundException(ref user_not_found) => {
                    SignInError::UserNotFound(user_not_found.message.clone())
                },
                _ => SignInError::SdkInitiateAuthError(Box::new(err)),
            },
            err => SignInError::SdkInitiateAuthError(Box::new(err)),
        })?;

        let session = initiate_auth_output.session.ok_or(SignInError::MissingSession)?;

        let challenge_name = initiate_auth_output
            .challenge_name
            .ok_or(SignInError::MissingChallengeName)?;

        let respond_to_auth_result = self
            .client
            .respond_to_auth_challenge()
            .client_id(client_id)
            .session(&session)
            .challenge_name(challenge_name)
            .challenge_responses("USERNAME", &self.username_or_email)
            .challenge_responses("ANSWER", "EMAIL_PASSWORDLESS_CODE")
            .client_metadata("CUSTOM_AUTH_FLOW", "EMAIL_PASSWORDLESS_CODE")
            .send()
            .await;

        let respond_to_auth_output =
            respond_to_auth_result.map_err(|err| SignInError::SdkRespondToAuthChallengeError(Box::new(err)))?;

        let session = respond_to_auth_output.session.ok_or(SignInError::MissingSession)?;

        let challenge_name = respond_to_auth_output
            .challenge_name
            .ok_or(SignInError::MissingChallengeName)?;

        let challenge_parameters = respond_to_auth_output
            .challenge_parameters
            .ok_or(SignInError::MissingChallengeParameters)?;

        Ok(SignInOutput {
            client: self.client,
            client_id: client_id.into(),
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
    #[error(transparent)]
    ValidationError(#[from] ValidationError),
    #[error(transparent)]
    SdkError(#[from] Box<SdkError<RespondToAuthChallengeError>>),
}

impl<'a> SignInOutput<'a> {
    pub async fn confirm(&mut self, code: impl Into<String>) -> Result<Credentials, SignInConfirmError> {
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
            SdkError::ServiceError { err: ref auth_err, .. } => match auth_err.kind {
                RespondToAuthChallengeErrorKind::UserLambdaValidationException(ref error) => {
                    match parse_lambda_error(error.clone()) {
                        Ok(err) => SignInConfirmError::ValidationError(err),
                        _ => SignInConfirmError::SdkError(Box::new(err)),
                    }
                },
                RespondToAuthChallengeErrorKind::NotAuthorizedException(_) => SignInConfirmError::NotAuthorized,
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
                false,
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
                },
                None => Err(SignInConfirmError::CouldNotSignIn),
            },
        }
    }
}

pub struct SignUpInput<'a> {
    client: &'a Client,
    client_id: Option<String>,
    email: String,
}

#[derive(Debug, Error)]
pub enum SignUpError {
    #[error(transparent)]
    SdkError(#[from] Box<SdkError<aws_sdk_cognitoidentityprovider::error::SignUpError>>),
    #[error(transparent)]
    ValidationError(#[from] ValidationError),
}

impl<'a> SignUpInput<'a> {
    pub fn new(client: &'a Client, email: impl Into<String>, client_id: impl Into<Option<String>>) -> Self {
        Self {
            client,
            client_id: client_id.into(),
            email: email.into(),
        }
    }

    pub async fn sign_up(self) -> Result<(), SignUpError> {
        let password = generate_password(32);
        let username = uuid::Uuid::new_v4().as_hyphenated().to_string();
        let client_id = self.client_id.as_ref().map_or(CLIENT_ID, String::as_str);

        let sign_up_result = self
            .client
            .sign_up()
            .client_id(client_id)
            .username(username)
            .password(&password)
            .user_attributes(AttributeType::builder().name("email").value(&self.email).build())
            .send()
            .await;

        sign_up_result.map_err(|err| match err {
            SdkError::ServiceError {
                err: ref sign_up_err, ..
            } => match sign_up_err.kind {
                SignUpErrorKind::UserLambdaValidationException(ref error) => match parse_lambda_error(error.clone()) {
                    Ok(err) => SignUpError::ValidationError(err),
                    _ => SignUpError::SdkError(Box::new(err)),
                },
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
    pub fn new(client: Client, username: impl Into<String>, access_token: impl Into<String>) -> Self {
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

fn rfc3339_deserialize_ignore_error<'de, D>(d: D) -> Result<Option<time::OffsetDateTime>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(time::serde::rfc3339::option::deserialize(d).ok().flatten())
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Credentials {
    pub email: Option<String>,
    pub access_token: Option<String>,
    pub id_token: Option<String>,
    pub refresh_token: Option<String>,
    #[serde(
        serialize_with = "time::serde::rfc3339::option::serialize",
        deserialize_with = "rfc3339_deserialize_ignore_error"
    )]
    pub expiration_time: Option<time::OffsetDateTime>,
    pub refresh_token_expired: Option<bool>,
}

#[derive(Debug, Error)]
pub enum RefreshError {
    #[error(transparent)]
    SdkError(#[from] Box<SdkError<aws_sdk_cognitoidentityprovider::error::InitiateAuthError>>),
    #[error("refresh token expired")]
    RefreshTokenExpired,
    #[error("refresh token not set")]
    RefreshTokenNotSet,
    #[error("empty authentication response")]
    EmptyAuthResponse,
}

impl Credentials {
    pub fn new(
        email: impl Into<String>,
        access_token: Option<String>,
        id_token: Option<String>,
        refresh_token: Option<String>,
        expires_in: i32,
        refresh_token_expired: bool,
    ) -> Self {
        Self {
            email: Some(email.into()),
            access_token,
            id_token,
            refresh_token,
            expiration_time: Some(time::OffsetDateTime::now_utc() + time::Duration::seconds(expires_in.into())),
            refresh_token_expired: Some(refresh_token_expired),
        }
    }

    pub fn save_credentials(&self) -> Result<()> {
        let data_dir = directories::fig_data_dir().map_err(|_| Error::Dir)?;

        if !data_dir.exists() {
            fs::create_dir_all(&data_dir)?;
        }

        let mut creds_file = File::create(data_dir.join("credentials.json"))?;

        #[cfg(unix)]
        {
            // Set permissions to 0600
            creds_file.set_permissions(std::os::unix::fs::PermissionsExt::from_mode(0o600))?;
        }

        serde_json::to_writer(&mut creds_file, self)?;

        #[cfg(target_os = "macos")]
        {
            use time::format_description::well_known::Rfc3339;

            use crate::{
                remove_default,
                set_default,
            };

            match &self.id_token {
                Some(id) => {
                    set_default("id_token", id)?;
                },
                None => {
                    remove_default("id_token").ok();
                },
            }

            match &self.access_token {
                Some(access) => {
                    set_default("access_token", access)?;
                },
                None => {
                    remove_default("access_token").ok();
                },
            }

            match &self.refresh_token {
                Some(refresh) => {
                    set_default("refresh_token", refresh)?;
                },
                None => {
                    remove_default("refresh_token").ok();
                },
            }

            match &self.email {
                Some(email) => {
                    set_default("userEmail", email)?;
                },
                None => {
                    remove_default("userEmail").ok();
                },
            }

            match &self.expiration_time {
                Some(time) => {
                    if let Ok(formatted_time) = time.format(&Rfc3339) {
                        set_default("expiration_time", formatted_time)?;
                    }
                },
                None => {
                    remove_default("expiration_time").ok();
                },
            }
        }

        Ok(())
    }

    pub fn load_credentials() -> Result<Credentials> {
        let data_dir = directories::fig_data_dir().map_err(|_| Error::Dir)?;

        let creds_path = data_dir.join("credentials.json");

        if !creds_path.exists() {
            return Err(Error::CredentialsFileNotExist);
        }

        let creds_file = File::open(data_dir.join("credentials.json"))?;

        Ok(serde_json::from_reader(creds_file)?)
    }

    pub async fn refresh_credentials(
        &mut self,
        client: &Client,
        client_id: Option<String>,
    ) -> Result<(), RefreshError> {
        if let Some(true) = self.refresh_token_expired {
            return Err(RefreshError::RefreshTokenExpired);
        }

        let refresh_token = self.refresh_token.as_ref().ok_or(RefreshError::RefreshTokenNotSet)?;
        let client_id = client_id.as_ref().map_or(CLIENT_ID, String::as_str);

        let out = match client
            .initiate_auth()
            .client_id(client_id)
            .auth_flow(AuthFlowType::RefreshTokenAuth)
            .auth_parameters("REFRESH_TOKEN", refresh_token)
            .send()
            .await
        {
            Ok(out) => out,
            Err(SdkError::ServiceError { err, .. }) if err.is_not_authorized_exception() => {
                self.refresh_token_expired = Some(true);
                self.save_credentials().ok();
                return Err(RefreshError::RefreshTokenExpired);
            },
            Err(err) => return Err(Box::new(err).into()),
        };

        match out.authentication_result {
            Some(auth_result) => {
                self.access_token = auth_result.access_token;
                self.id_token = auth_result.id_token;
                self.expiration_time =
                    Some(time::OffsetDateTime::now_utc() + time::Duration::seconds(auth_result.expires_in.into()));
                self.refresh_token_expired = Some(false);
            },
            None => return Err(RefreshError::EmptyAuthResponse),
        }

        Ok(())
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

    pub fn is_expired_epslion(&self, epsilon: time::Duration) -> bool {
        match self.get_expiration_time() {
            Some(expiration_time) => expiration_time + epsilon < time::OffsetDateTime::now_utc(),
            None => true,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.is_expired_epslion(time::Duration::seconds(30))
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
