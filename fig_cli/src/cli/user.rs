use std::process::exit;

use anyhow::{
    bail,
    Result,
};
use clap::Subcommand;
use crossterm::style::Stylize;
use fig_auth::cognito::{
    get_client,
    Credentials,
    SignInConfirmError,
    SignInError,
    SignInInput,
    SignUpInput,
};
use reqwest::Method;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::{
    json,
    Value,
};
use time::format_description::well_known::Rfc3339;

use super::OutputFormat;
use crate::cli::dialoguer_theme;
use crate::util::api::request;

#[derive(Subcommand, Debug)]
pub enum RootUserSubcommand {
    /// Login to Fig
    Login {
        /// Manually refresh the auth token
        #[clap(long, short, action)]
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
    Whoami {
        /// Output format to use
        #[clap(long, short, value_enum, action, default_value_t)]
        format: OutputFormat,
        /// Only print the user's email address, this is quicker since it doesn't require a network
        /// request
        #[clap(long, short = 'e', action)]
        only_email: bool,
    },
    #[clap(subcommand)]
    Tokens(TokensSubcommand),
}

impl UserSubcommand {
    pub async fn execute(self) -> Result<()> {
        match self {
            Self::Root(cmd) => cmd.execute().await,
            Self::Whoami { format, only_email } => whoami_cli(format, only_email).await,
            Self::Tokens(cmd) => cmd.execute().await,
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum TokensSubcommand {
    New {
        /// The name of the token
        #[clap(long, action)]
        name: String,
        /// The expiration date of the token in RFC3339 format
        #[clap(long, action, conflicts_with = "expires-in")]
        expires_date: Option<String>,
        /// The time till the token expires (e.g. "90d")
        #[clap(long, action, conflicts_with = "expires-date")]
        expires_in: Option<String>,
        /// The team namespace to create the token for
        #[clap(long, short, action)]
        team: Option<String>,
    },
    List {
        /// The team namespace to list the tokens for
        #[clap(long, short, action, conflicts_with = "personal")]
        team: Option<String>,
        /// Only list tokens owned by the current user
        #[clap(long, short, action, conflicts_with = "team")]
        personal: bool,
        #[clap(long, short, value_enum, action, default_value_t)]
        format: OutputFormat,
    },
    Revoke {
        /// The name of the token to revoke
        #[clap(long, action)]
        name: String,
        /// The team namespace to revoke the token for
        #[clap(long, short, action)]
        team: Option<String>,
    },
    /// Validate a token is valid
    Validate {
        /// The token to validate
        #[clap(long, action)]
        token: String,
    },
}

impl TokensSubcommand {
    pub async fn execute(self) -> Result<()> {
        match self {
            Self::New {
                name,
                expires_date,
                expires_in,
                team,
            } => {
                let expires_at = match (expires_date, expires_in) {
                    (Some(expires_date), None) => match time::OffsetDateTime::parse(&expires_date, &Rfc3339) {
                        Ok(date) => {
                            println!("{date}");
                            Some(date)
                        },
                        Err(err) => {
                            bail!("Failed to parse date: {err}");
                        },
                    },
                    (None, Some(expires_in)) => {
                        let duration = humantime::parse_duration(&expires_in)?;
                        Some(time::OffsetDateTime::now_utc() + duration)
                    },
                    (None, None) => None,
                    (Some(_), Some(_)) => {
                        bail!("You can only specify one of --expires-date or --expires-in");
                    },
                }
                .and_then(|date| date.format(&Rfc3339).ok());

                let json: Value = request(
                    Method::POST,
                    "/auth/tokens/new",
                    Some(&json!({ "name": name, "team": team, "expiresAt": expires_at })),
                    true,
                )
                .await?;

                match json.get("token").and_then(|x| x.as_str()) {
                    Some(val) => {
                        eprintln!("API token:");
                        println!("{val}");
                    },
                    None => {
                        bail!("Could not get tokens: {json}");
                    },
                }
                Ok(())
            },
            Self::List { format, team, personal } => {
                let json: Value = request(
                    Method::GET,
                    "/auth/tokens/list",
                    Some(&json!({ "team": team, "personal": personal })),
                    true,
                )
                .await?;

                match json.get("tokens") {
                    Some(val) => match format {
                        OutputFormat::Json => {
                            println!("{}", serde_json::to_string(val).unwrap())
                        },
                        OutputFormat::JsonPretty => {
                            println!("{}", serde_json::to_string_pretty(val).unwrap())
                        },
                        OutputFormat::Plain => {
                            if let Some(tokens) = val.as_array() {
                                if tokens.is_empty() {
                                    eprintln!("No tokens");
                                } else {
                                    println!(
                                        "{}",
                                        format!("{name:<20}{namespace}", name = "Name", namespace = "Namespace").bold()
                                    );
                                    for token in tokens {
                                        let name = token["name"].as_str().unwrap_or_default();
                                        let namespace = token["namespace"]["username"].as_str().unwrap_or_default();
                                        println!("{name:<20}{namespace}");
                                    }
                                }
                            } else {
                                bail!("Tokens is not an array: {json}");
                            }
                        },
                    },
                    None => {
                        bail!("Could not get tokens: {json}");
                    },
                }
                Ok(())
            },
            Self::Revoke { name, team } => {
                request::<Value, _, _>(
                    Method::POST,
                    "/auth/tokens/revoke",
                    Some(&json!({ "name": name, "team": team })),
                    true,
                )
                .await?;

                match team {
                    Some(team) => {
                        println!("Revoked token {name} for team {team}");
                    },
                    None => {
                        println!("Revoked token {name}");
                    },
                }
                Ok(())
            },
            Self::Validate { token } => {
                let valid: Value = request(
                    Method::POST,
                    "/auth/tokens/validate",
                    Some(&json!({ "token": token })),
                    true,
                )
                .await?;

                if let Some(&Value::String(ref username)) = valid.get("username") {
                    println!("{username}");
                    Ok(())
                } else {
                    exit(1);
                }
            },
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
                SignUpInput::new(&client, client_id, email).sign_up().await?;

                sign_in_input.sign_in().await?
            },
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
                request::<Value, _, _>(Method::POST, "/user/login", None, true).await?;
                println!("Login successful!");
                return Ok(());
            },
            Err(err) => match err {
                SignInConfirmError::ErrorCodeMismatch => {
                    println!("Code mismatch, try again...");
                    continue;
                },
                SignInConfirmError::NotAuthorized => {
                    return Err(anyhow::anyhow!(
                        "Not authorized, you may have entered the wrong code too many times."
                    ));
                },
                err => return Err(err.into()),
            },
        };
    }
}

// Logout from fig
pub async fn logout_cli() -> Result<()> {
    let mut creds = Credentials::load_credentials()?;
    creds.clear_credentials();
    creds.save_credentials()?;

    #[cfg(target_os = "macos")]
    {
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
    }

    println!("Logged out");
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WhoamiResponse {
    email: String,
    username: Option<String>,
}

pub async fn whoami_cli(format: OutputFormat, only_email: bool) -> Result<()> {
    let email = fig_auth::get_email();

    match email {
        Some(email) => {
            if only_email {
                match format {
                    OutputFormat::Plain => println!("Email: {}", email),
                    OutputFormat::Json => println!("{}", serde_json::to_string(&json!({ "email": email }))?),
                    OutputFormat::JsonPretty => {
                        println!("{}", serde_json::to_string_pretty(&json!({ "email": email }))?)
                    },
                }
            } else {
                let response: WhoamiResponse = request(Method::GET, "/user/whoami", None, true).await?;
                match format {
                    OutputFormat::Plain => match response.username {
                        Some(username) => println!("Email: {}\nUsername: {}", response.email, username),
                        None => println!("Email: {}\nUsername is null", response.email),
                    },
                    OutputFormat::Json => println!("{}", serde_json::to_string(&response)?),
                    OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&response)?),
                }
            }
            Ok(())
        },
        None => {
            match format {
                OutputFormat::Plain => println!("Not logged in"),
                OutputFormat::Json => println!("{}", serde_json::to_string(&json!({ "email": null }))?),
                OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&json!({ "email": null }))?),
            }
            exit(1);
        },
    }
}
