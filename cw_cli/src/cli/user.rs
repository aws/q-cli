use std::fmt;
use std::fmt::Display;
use std::process::exit;
use std::time::Duration;

use auth::builder_id::{
    poll_create_token,
    start_device_authorization,
    BuilderIdToken,
    PollCreateToken,
    TokenType,
};
use auth::secret_store::SecretStore;
use clap::Subcommand;
use crossterm::style::Stylize;
use eyre::Result;
use fig_ipc::local::{
    login_command,
    logout_command,
};
use serde_json::json;
use tracing::error;

use super::OutputFormat;
use crate::util::spinner::{
    Spinner,
    SpinnerComponent,
};
use crate::util::{
    choose,
    input,
};

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum RootUserSubcommand {
    /// Login to CodeWhisperer
    Login,
    /// Logout of CodeWhisperer
    Logout,
    /// Prints details about the current user
    Whoami {
        /// Output format to use
        #[arg(long, short, value_enum, default_value_t)]
        format: OutputFormat,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AuthMethod {
    Email,
    IdentityCenter,
}

impl Display for AuthMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthMethod::Email => write!(f, "Sing up or Sign in with Email (Builder ID)"),
            AuthMethod::IdentityCenter => write!(f, "Sign in with IAM Identity Center"),
        }
    }
}

impl RootUserSubcommand {
    pub async fn execute(self) -> Result<()> {
        match self {
            Self::Login => {
                if auth::is_logged_in().await {
                    eyre::bail!("Already logged in, please logout with `cw logout` first");
                }

                let options = [AuthMethod::Email, AuthMethod::IdentityCenter];
                let login_method = options[choose("Select action", &options)?];
                match login_method {
                    AuthMethod::Email | AuthMethod::IdentityCenter => {
                        let (start_url, region) = match login_method {
                            AuthMethod::Email => (None, None),
                            AuthMethod::IdentityCenter => {
                                let start_url = input("Enter Start URL")?;
                                let region = input("Enter Region")?;
                                (Some(start_url), Some(region))
                            },
                        };

                        let secret_store = SecretStore::load().await?;
                        let device_auth =
                            start_device_authorization(&secret_store, start_url.clone(), region.clone()).await?;

                        println!();
                        println!("Confirm the following code in the browser");
                        println!("Code: {}", device_auth.user_code.bold());
                        println!();
                        // confirm("Continue?")?;

                        if fig_util::open_url_async(&device_auth.verification_uri_complete)
                            .await
                            .is_err()
                        {
                            println!("Open this URL: {}", device_auth.verification_uri_complete);
                        };
                        // println!();

                        let mut spinner = Spinner::new(vec![
                            SpinnerComponent::Spinner,
                            SpinnerComponent::Text(" Logging in...".into()),
                        ]);

                        loop {
                            tokio::time::sleep(Duration::from_secs(device_auth.interval.try_into().unwrap_or(1))).await;
                            match poll_create_token(
                                &secret_store,
                                device_auth.device_code.clone(),
                                start_url.clone(),
                                region.clone(),
                            )
                            .await
                            {
                                PollCreateToken::Pending => {},
                                PollCreateToken::Complete(_) => {
                                    spinner.stop_with_message("Logged in successfully".into());
                                    break;
                                },
                                PollCreateToken::Error(err) => {
                                    spinner.stop();
                                    return Err(err.into());
                                },
                            };
                        }
                    },
                    // Other methods soon!
                };

                if let Err(err) = login_command().await {
                    error!(%err, "Failed to send login command");
                }

                Ok(())
            },
            Self::Logout => {
                // let telem_join = tokio::spawn(fig_telemetry::emit_track(TrackEvent::new(
                //     TrackEventType::Logout,
                //     TrackSource::Cli,
                //     env!("CARGO_PKG_VERSION").into(),
                //     empty::<(&str, &str)>(),
                // )));

                let logout_join = logout_command();

                let (_, _) = tokio::join!(logout_join, auth::logout());

                println!("You are now logged out");
                println!("Run {} to log back in to CodeWhisperer", "cw login".magenta());
                Ok(())
            },
            Self::Whoami { format } => {
                let secret_store = SecretStore::load().await?;
                let builder_id = BuilderIdToken::load(&secret_store).await;

                match builder_id {
                    Ok(Some(token)) => {
                        format.print(
                            || match token.token_type() {
                                TokenType::BuilderId => "Logged in with Builder ID".into(),
                                TokenType::IamIdentityCenter => {
                                    format!(
                                        "Logged in with IAM Identity Center ({})",
                                        token.start_url.as_ref().unwrap()
                                    )
                                },
                            },
                            || {
                                json!({
                                    "accountType": match token.token_type() {
                                        TokenType::BuilderId => "BuilderId",
                                        TokenType::IamIdentityCenter => "IamIdentityCenter",
                                    },
                                    "startUrl": token.start_url,
                                    "region": token.region,
                                })
                            },
                        );
                        Ok(())
                    },
                    _ => {
                        format.print(|| "Not logged in", || json!({ "account": null }));
                        exit(1);
                    },
                }
            },
        }
    }
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum UserSubcommand {
    #[command(flatten)]
    Root(RootUserSubcommand),
}

impl UserSubcommand {
    pub async fn execute(self) -> Result<()> {
        match self {
            Self::Root(cmd) => cmd.execute().await,
        }
    }
}
