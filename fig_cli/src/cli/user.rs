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
use serde_json::{
    json,
    Value,
};
use time::format_description::well_known::Rfc3339;

use super::OutputFormat;
use crate::cli::util::dialoguer_theme;
use crate::util::api::request;

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
    Tokens(TokensSubcommand),
}

impl UserSubcommand {
    pub async fn execute(self) -> Result<()> {
        match self {
            Self::Root(cmd) => cmd.execute().await,
            Self::Whoami => whoami_cli().await,
            Self::Tokens(cmd) => cmd.execute().await,
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum TokensSubcommand {
    New {
        /// The name of the token
        name: String,
        /// The expiration date of the token in RFC3339 format
        #[clap(long, conflicts_with = "expires-in")]
        expires_date: Option<String>,
        /// The time till the token expires (e.g. "90d")
        #[clap(long, conflicts_with = "expires-date")]
        expires_in: Option<String>,
        /// The team namespace to create the token for
        #[clap(long, short)]
        team: Option<String>,
    },
    List {
        /// The team namespace to list the tokens for
        #[clap(long, short, conflicts_with = "personal")]
        team: Option<String>,
        /// Only list tokens owned by the current user
        #[clap(long, short, conflicts_with = "team")]
        personal: bool,
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

                let json: serde_json::Value = request(
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
                let _json: Value = request(
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

pub async fn whoami_cli() -> Result<()> {
    match fig_auth::get_email() {
        Some(email) => {
            println!("Logged in as {}", email);
            Ok(())
        },
        None => bail!("Not logged in"),
    }
}
