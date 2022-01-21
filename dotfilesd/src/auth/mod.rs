use anyhow::Result;
use aws_sdk_cognitoidentityprovider::{
    model::{AttributeType, AuthFlowType, ChallengeNameType},
    AppName, Client, Config, Region,
};
use rand::Rng;
use std::{borrow::Cow, collections::HashMap};

pub fn get_client(client_name: impl Into<Cow<'static, str>>) -> Result<Client> {
    let config = Config::builder()
        .app_name(AppName::new(client_name)?)
        .region(Region::new("us-east-1"))
        .build();

    Ok(Client::from_conf(config))
}

pub struct Credentials {
    access_token: Option<String>,
    id_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: i32,
}

pub struct SignInInput {
    client: Client,
    client_id: String,
    username_or_email: String,
}

impl SignInInput {
    pub fn new(client: Client, client_id: impl Into<String>, username_or_email: impl Into<String>) -> Self {
        Self {
            client,
            client_id: client_id.into(),
            username_or_email: username_or_email.into(),
        }
    }

    pub async fn sign_in(self) -> Result<SignInOutput> {
        let output = self
            .client
            .initiate_auth()
            .client_id(&self.client_id)
            .auth_flow(AuthFlowType::CustomAuth)
            .auth_parameters("USERNAME", &self.username_or_email)
            .client_metadata("CUSTOM_AUTH", "PASSWORDLESS_EMAIL")
            .send()
            .await?;

        // TODO: handle error
        let challenge_name = match output.challenge_name {
            Some(challenge_name) => challenge_name,
            None => return Err(anyhow::anyhow!("Challenge name is None")),
        };

        Ok(SignInOutput {
            client: self.client,
            client_id: self.client_id,
            username_or_email: self.username_or_email,
            session: output.session,
            challenge_name: challenge_name,
            challenge_parameters: output.challenge_parameters,
        })
    }
}

pub struct SignInOutput {
    client: Client,
    client_id: String,
    username_or_email: String,
    session: Option<String>,
    challenge_name: ChallengeNameType,
    challenge_parameters: Option<HashMap<String, String>>,
}

impl SignInOutput {
    pub async fn confirm(&mut self, code: String) -> Result<Credentials> {
        let out = self
            .client
            .respond_to_auth_challenge()
            .client_id(&self.client_id)
            .session(&self.session.clone().unwrap_or_default())
            .challenge_name(self.challenge_name.clone())
            .challenge_responses("USERNAME", &self.username_or_email)
            .challenge_responses("ANSWER", &code)
            .send()
            .await?;

        // TODO: handle error

        match out.authentication_result {
            Some(auth_result) => Ok(Credentials {
                access_token: auth_result.access_token,
                id_token: auth_result.id_token,
                refresh_token: auth_result.refresh_token,
                expires_in: auth_result.expires_in,
            }),
            None => match out.session {
                Some(session) => {
                    self.session = Some(session);
                    self.challenge_name = out.challenge_name.unwrap();
                    self.challenge_parameters = out.challenge_parameters;
                    Err(anyhow::anyhow!("Challenge name is None"))
                }
                None => Err(anyhow::anyhow!("Could not sign in")),
            },
        }
    }
}

struct ChangeUsernameInput {
    client: Client,
    username: String,
    access_token: String,
}

pub struct SignUpInput {
    client: Client,
    client_id: String,
    email: String,
}

impl SignUpInput {
    pub fn new(client: Client, client_id: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            client,
            client_id: client_id.into(),
            email: email.into(),
        }
    }

    pub async fn sign_up(self) -> Result<SignUpOutput> {
        // Generate password
        let password = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .map(|c| c as char)
            .take(30)
            .collect::<String>();

        // Generate uuid
        let username = uuid::Uuid::new_v4().to_hyphenated().to_string();

        let out = self
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
            .await?;

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

pub struct SignUpOutput {
    client: Client,
    client_id: String,
    username: String,
    password: String,
    user_sub: Option<String>,
    user_confirmed: bool,
}

impl SignUpOutput {
    async fn confirm(&mut self, code: impl Into<String>) -> Result<Credentials> {
        self.client
            .confirm_sign_up()
            .client_id(&self.client_id)
            .username(&self.username)
            .confirmation_code(code)
            .send()
            .await?;

        // TODO: handle error

        let out = self
            .client
            .initiate_auth()
            .client_id(&self.client_id)
            .auth_flow(AuthFlowType::UserPasswordAuth)
            .auth_parameters("USERNAME", &self.username)
            .auth_parameters("PASSWORD", &self.password)
            .send()
            .await?;

        // TODO: handle error

        match out.authentication_result {
            Some(auth_result) => Ok(Credentials {
                access_token: auth_result.access_token,
                id_token: auth_result.id_token,
                refresh_token: auth_result.refresh_token,
                expires_in: auth_result.expires_in,
            }),
            None => Err(anyhow::anyhow!("Could not sign in")),
        }
    }

    async fn resend(&self) -> Result<()> {
        self.client
            .resend_confirmation_code()
            .client_id(&self.client_id)
            .username(&self.username)
            .send()
            .await?;

        Ok(())
    }
}
