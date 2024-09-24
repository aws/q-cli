//! A module for making working with the app easier by grabbing credentials via ada for sigv4

use aws_credential_types::Credentials;
use aws_credential_types::provider::error::CredentialsError;
use aws_credential_types::provider::{
    ProvideCredentials,
    Result as CredsResult,
    future,
};
use fig_util::directories::home_dir;
use tokio::process::Command;
use tracing::debug;

struct AccountAndRole {
    account: String,
    role: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AdaOutput {
    access_key_id: String,
    secret_access_key: String,
    session_token: String,
    #[serde(with = "time::serde::rfc3339")]
    expiration: time::OffsetDateTime,
}

pub struct AdaCredentialsProviderBuilder {
    _inner: (),
}

impl AdaCredentialsProviderBuilder {
    pub fn build(self) -> AdaCredentialsProvider {
        AdaCredentialsProvider { _inner: () }
    }
}

#[derive(Debug)]
pub struct AdaCredentialsProvider {
    _inner: (),
}

impl AdaCredentialsProvider {
    const NAME: &'static str = "AdaCredentialsProvider";

    pub fn builder() -> AdaCredentialsProviderBuilder {
        AdaCredentialsProviderBuilder { _inner: () }
    }

    fn load_account_and_role() -> Option<AccountAndRole> {
        match (std::env::var("ADA_ACCOUNT"), std::env::var("ADA_ROLE")) {
            (Ok(account), Ok(role)) => Some(AccountAndRole { account, role }),
            _ => None,
        }
    }

    async fn credentials(&self) -> CredsResult {
        let Some(AccountAndRole { account, role }) = Self::load_account_and_role() else {
            debug!("No account and role found for Ada");
            return Err(CredentialsError::provider_error("No account and role found"));
        };

        let ada_exe = which::which("ada").unwrap_or_else(|_| {
            home_dir()
                .expect("failed to get home dir")
                .join(".toolbox")
                .join("bin")
                .join("ada")
        });

        let mut command = Command::new(ada_exe);
        command.args([
            "credentials",
            "print",
            "--account",
            account.as_str(),
            "--role",
            role.as_str(),
            "--format",
            "json",
        ]);

        let output = command
            .output()
            .await
            .map_err(|err| CredentialsError::provider_error(err))?;

        debug!(status =% output.status, "Loaded credentials from ada");

        let AdaOutput {
            access_key_id,
            secret_access_key,
            session_token,
            expiration,
        } = serde_json::from_slice(&output.stdout).map_err(|err| CredentialsError::provider_error(err))?;

        Ok(Credentials::new(
            access_key_id,
            secret_access_key,
            Some(session_token),
            Some(expiration.into()),
            Self::NAME,
        ))
    }
}

impl ProvideCredentials for AdaCredentialsProvider {
    fn provide_credentials<'a>(&'a self) -> future::ProvideCredentials<'a>
    where
        Self: 'a,
    {
        future::ProvideCredentials::new(self.credentials())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ada_credentials_provider() {
        let provider = AdaCredentialsProvider::builder().build();
        let credentials = provider.provide_credentials().await.unwrap();
        println!("{:?}", credentials);
    }
}
