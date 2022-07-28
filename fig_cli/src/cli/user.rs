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
use fig_request::Request;
use fig_settings::state;
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

#[derive(Subcommand, Debug)]
pub enum RootUserSubcommand {
    /// Login to Fig
    Login {
        /// Refresh the auth token if expired
        #[clap(long, short, value_parser)]
        refresh: bool,
        /// Force a refresh of the auth token
        #[clap(long, value_parser)]
        hard_refresh: bool,
    },
    /// Logout of Fig
    Logout,
}

impl RootUserSubcommand {
    pub async fn execute(self) -> Result<()> {
        match self {
            Self::Login { refresh, hard_refresh } => login_cli(refresh, hard_refresh).await,
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
        #[clap(long, short, value_enum, value_parser, default_value_t)]
        format: OutputFormat,
        /// Only print the user's email address, this is quicker since it doesn't require a network
        /// request
        #[clap(long, short = 'e', value_parser)]
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
        #[clap(value_parser)]
        name: String,
        /// The expiration date of the token in RFC3339 format
        #[clap(long, value_parser, conflicts_with = "expires-in")]
        expires_date: Option<String>,
        /// The time till the token expires (e.g. "90d")
        #[clap(long, value_parser, conflicts_with = "expires-date")]
        expires_in: Option<String>,
        /// The team namespace to create the token for
        #[clap(long, short, value_parser)]
        team: String,
    },
    List {
        /// The team namespace to list the tokens for
        #[clap(long, short, value_parser)]
        team: String,
        #[clap(long, short, value_enum, value_parser, default_value_t)]
        format: OutputFormat,
    },
    Revoke {
        /// The name of the token to revoke
        #[clap(value_parser)]
        name: String,
        /// The team namespace to revoke the token for
        #[clap(long, short, value_parser)]
        team: String,
    },
    /// Validate a token is valid
    Validate {
        /// The token to validate
        #[clap(value_parser)]
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

                let json = Request::post("/auth/tokens/new")
                    .auth()
                    .body(json!({ "name": name, "team": team, "expiresAt": expires_at }))
                    .json()
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
            Self::List { format, team } => {
                let json = Request::get("/auth/tokens/list")
                    .auth()
                    .body(json!({ "namespace": team }))
                    .json()
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
                Request::post("/auth/tokens/revoke")
                    .auth()
                    .body(json!({ "team": team }))
                    .send()
                    .await?;

                println!("Revoked token {name} for team {team}");

                Ok(())
            },
            Self::Validate { token } => {
                let valid = Request::post("/auth/tokens/validate")
                    .auth()
                    .body(json!({ "token": token }))
                    .json()
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
pub async fn login_cli(refresh: bool, hard_refresh: bool) -> Result<()> {
    let client = get_client()?;

    if refresh || hard_refresh {
        let mut creds = Credentials::load_credentials()?;
        if creds.is_expired() || hard_refresh {
            creds.refresh_credentials(&client, None).await?;
            creds.save_credentials()?;
        }
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

    let sign_in_input = SignInInput::new(&client, trimmed_email, None);

    println!("Sending login code to {trimmed_email}...");
    println!("Please check your email for the code");

    let mut sign_in_output = match sign_in_input.sign_in().await {
        Ok(out) => out,
        Err(err) => match err {
            SignInError::UserNotFound(_) => {
                SignUpInput::new(&client, email, None).sign_up().await?;

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
                let body = match state::get_string("anonymousId") {
                    Ok(Some(anonymous_id)) => json!({ "anonymousId": anonymous_id }),
                    _ => json!({}),
                };
                Request::post("/user/login").auth().body(body).send().await?;
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
        tokio::process::Command::new("defaults")
            .args(["delete", "com.mschrage.fig.shared"])
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
                    OutputFormat::Plain => println!("Email: {email}"),
                    OutputFormat::Json => println!("{}", serde_json::to_string(&json!({ "email": email }))?),
                    OutputFormat::JsonPretty => {
                        println!("{}", serde_json::to_string_pretty(&json!({ "email": email }))?)
                    },
                }
            } else {
                let response: WhoamiResponse = Request::get("/user/whoami").auth().deser_json().await?;
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
