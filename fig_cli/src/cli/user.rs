use crate::{cli::util::dialoguer_theme, util::api::handle_fig_response};

use anyhow::{bail, Result};
use clap::Subcommand;
use crossterm::style::Stylize;
use fig_auth::{
    cognito::{get_client, Credentials, SignInConfirmError, SignInError, SignInInput, SignUpInput},
    get_token,
};
use fig_settings::api_host;
use serde_json::json;
use std::process::exit;

use super::OutputFormat;

#[derive(Subcommand, Debug)]
pub enum RootUserSubcommand {
    /// Login to Fig
    Login {
        /// Manually refresh the auth token
        #[clap(long, short)]
        refresh: bool,
    },
    /// Logout of Fig
    Logout,
}

impl RootUserSubcommand {
    pub async fn execute(self) -> Result<()> {
        match self {
            Self::Login { refresh } => login_cli(refresh).await,
            Self::Logout => logout_cli().await,
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum UserSubcommand {
    #[clap(flatten)]
    Root(RootUserSubcommand),
    Whoami,
    #[clap(subcommand)]
    Token(TokenSubcommand),
}

impl UserSubcommand {
    pub async fn execute(self) -> Result<()> {
        match self {
            Self::Root(cmd) => cmd.execute().await,
            Self::Whoami => whoami_cli().await,
            Self::Token(cmd) => cmd.execute().await,
        }
    }
}

/*
fig user token new --name <name> --expires <date> [ --team <namespace> ]

fig user token list [ --team <namespace> ]

fig user token revoke <token-name> [ --team <namespace> ]
 */

#[derive(Subcommand, Debug)]
pub enum TokenSubcommand {
    New {
        /// The name of the token
        name: String,
        /// The expiration date of the token
        #[clap(long, short)]
        expires: Option<String>,
        /// The team namespace to create the token for
        #[clap(long, short)]
        team: Option<String>,
    },
    List {
        /// The team namespace to list the tokens for
        #[clap(long, short)]
        team: Option<String>,
        #[clap(long, short, arg_enum, default_value_t)]
        format: OutputFormat,
    },
    Revoke {
        /// The name of the token to revoke
        name: String,
        /// The team namespace to revoke the token for
        #[clap(long, short)]
        team: Option<String>,
    },
}

impl TokenSubcommand {
    pub async fn execute(self) -> Result<()> {
        match self {
            Self::New {
                name,
                expires,
                team: _,
            } => {
                println!("Creating token \"{name}\"");

                if let Some(expires) = expires {
                    match time::OffsetDateTime::parse(
                        &expires,
                        &time::format_description::well_known::Rfc3339,
                    ) {
                        Ok(date) => {
                            println!("{date}");
                        }
                        Err(err) => {
                            println!("Failed to parse date: {err}");
                        }
                    }
                }

                let api_host = api_host();
                let token = get_token().await.unwrap();

                let url = reqwest::Url::parse(&format!("{api_host}/auth/tokens/new")).unwrap();
                let response = reqwest::Client::new()
                    .post(url)
                    .bearer_auth(token)
                    .header("Accept", "application/json")
                    .json(&json!({ "name": name }))
                    .send()
                    .await?;

                let json: serde_json::Value = handle_fig_response(response)
                    .await?
                    .json()
                    .await?;

                match json.get("apiToken").and_then(|x| x.as_str()) {
                    Some(val) => {
                        eprintln!("API token:");
                        println!("{val}");
                    }
                    None => {
                        eprintln!("Could not get API token");
                        exit(1);
                    }
                }
                Ok(())
            }
            Self::List { format, team: _ } => {
                let api_host = api_host();
                let token = get_token().await.unwrap();

                let url = reqwest::Url::parse(&format!("{api_host}/auth/tokens/list")).unwrap();
                let response = reqwest::Client::new()
                    .get(url)
                    .bearer_auth(token)
                    .header("Accept", "application/json")
                    .send()
                    .await?;

                let json: serde_json::Value = handle_fig_response(response)
                    .await?
                    .json()
                    .await?;

                match json.get("apiTokens") {
                    Some(val) => {
                        match format {
                            OutputFormat::Json => {
                                println!("{}", serde_json::to_string(val).unwrap())
                            }
                            OutputFormat::JsonPretty => {
                                println!("{}", serde_json::to_string_pretty(val).unwrap())
                            }
                            OutputFormat::Plain => {
                                todo!();
                            }
                        }
                    }
                    None => {
                        eprintln!("Could not get API token");
                        exit(1);
                    }
                }
                Ok(())
            }
            Self::Revoke { name, team: _ } => {
                let api_host = api_host();
                let token = get_token().await.unwrap();

                let url = reqwest::Url::parse(&format!("{api_host}/auth/tokens/revoke")).unwrap();
                let response = reqwest::Client::new()
                    .post(url)
                    .bearer_auth(token)
                    .header("Accept", "application/json")
                    .json(&json!({ "name": name }))
                    .send()
                    .await?;

                handle_fig_response(response).await?;
                
                println!("Revoked token: {name}");
                Ok(())
            }
        }
    }
}

/// Login to fig
pub async fn login_cli(refresh: bool) -> Result<()> {
    let client_id = "hkinciohdp1i7h0imdk63a4bv";
    let client = get_client()?;

    if refresh {
        let mut creds = Credentials::load_credentials()?;
        creds.refresh_credentials(&client, client_id).await?;
        creds.save_credentials()?;
        return Ok(());
    }

    println!("{}", "Login to Fig".bold().magenta());

    let theme = dialoguer_theme();

    let email: String = dialoguer::Input::with_theme(&theme)
        .with_prompt("Email")
        .validate_with(|input: &String| -> Result<(), &str> {
            if validator::validate_email(input.trim()) {
                Ok(())
            } else {
                Err("This is not a valid email")
            }
        })
        .interact_text()?;

    let trimmed_email = email.trim();

    let sign_in_input = SignInInput::new(&client, client_id, trimmed_email);

    println!("Sending login code to {}...", trimmed_email);
    println!("Please check your email for the code");

    let mut sign_in_output = match sign_in_input.sign_in().await {
        Ok(out) => out,
        Err(err) => match err {
            SignInError::UserNotFound(_) => {
                SignUpInput::new(&client, client_id, email)
                    .sign_up()
                    .await?;

                sign_in_input.sign_in().await?
            }
            err => return Err(err.into()),
        },
    };

    loop {
        let login_code: String = dialoguer::Input::with_theme(&theme)
            .with_prompt("Login code")
            .validate_with(|input: &String| -> Result<(), &str> {
                if input.len() == 6 && input.chars().all(|c| c.is_ascii_digit()) {
                    Ok(())
                } else {
                    Err("Code must be 6 digits")
                }
            })
            .interact_text()?;

        match sign_in_output.confirm(login_code.trim()).await {
            Ok(creds) => {
                creds.save_credentials()?;
                println!("Login successful!");
                return Ok(());
            }
            Err(err) => match err {
                SignInConfirmError::ErrorCodeMismatch => {
                    println!("Code mismatch, try again...");
                    continue;
                }
                SignInConfirmError::NotAuthorized => {
                    return Err(anyhow::anyhow!(
                        "Not authorized, you may have entered the wrong code too many times."
                    ));
                }
                err => return Err(err.into()),
            },
        };
    }
}

// Logout from fig
pub async fn logout_cli() -> Result<()> {
    let mut creds = Credentials::load_credentials()?;
    creds.clear_cridentials();
    creds.save_credentials()?;

    let uuid = fig_auth::get_default("uuid").unwrap_or_default();
    tokio::process::Command::new("defaults")
        .args(["delete", "com.mschrage.fig"])
        .output()
        .await
        .ok();
    tokio::process::Command::new("defaults")
        .args(["delete", "com.mschrage.fig.shared"])
        .output()
        .await
        .ok();
    tokio::process::Command::new("defaults")
        .args(["write", "com.mschrage.fig", "uuid", &uuid])
        .output()
        .await
        .ok();

    println!("Logged out");

    Ok(())
}

pub async fn whoami_cli() -> Result<()> {
    match fig_auth::get_email() {
        Some(email) => println!("Logged in as {}", email),
        None => bail!("Not logged in"),
    }

    Ok(())
}
