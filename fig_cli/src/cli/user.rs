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
        /// Email to login to
        #[clap(value_parser)]
        email: Option<String>,
        ///
        #[clap(long, value_parser, hide = true)]
        switchable: bool,
    },
    /// Logout of Fig
    Logout,
}

impl RootUserSubcommand {
    pub async fn execute(self) -> Result<()> {
        match self {
            Self::Login {
                email,
                refresh,
                hard_refresh,
                switchable,
            } => {
                let client = get_client()?;

                if refresh || hard_refresh {
                    let mut creds = Credentials::load_credentials()?;
                    if creds.is_expired() || hard_refresh {
                        creds.refresh_credentials(&client, None).await?;
                        creds.save_credentials()?;
                    }
                    return Ok(());
                }

                if email.is_none() {
                    println!("{}", "Login to Fig".bold().magenta());
                }

                let email: String = match email {
                    Some(email) => email,
                    None => dialoguer::Input::with_theme(&dialoguer_theme())
                        .with_prompt("Email")
                        .validate_with(|input: &String| -> Result<(), &str> {
                            if validator::validate_email(input.trim()) {
                                Ok(())
                            } else {
                                Err("This is not a valid email")
                            }
                        })
                        .interact_text()?,
                };

                let trimmed_email = email.trim();
                let sign_in_input = SignInInput::new(&client, trimmed_email, None);

                println!("Sending login code to {trimmed_email}...");
                println!("Please check your email for the code");

                let mut sign_in_output = match sign_in_input.sign_in().await {
                    Ok(out) => out,
                    Err(err) => match err {
                        SignInError::UserNotFound(_) => {
                            SignUpInput::new(&client, &email, None).sign_up().await?;
                            sign_in_input.sign_in().await?
                        },
                        err => return Err(err.into()),
                    },
                };

                loop {
                    let login_code: String = dialoguer::Input::with_theme(&dialoguer_theme())
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

                            if switchable {
                                let dir = Credentials::account_credentials_dir()?;
                                if !dir.exists() {
                                    std::fs::create_dir_all(&dir)?;
                                }
                                std::fs::copy(
                                    Credentials::path()?,
                                    Credentials::account_credentials_path(&trimmed_email)?,
                                )?;
                            }

                            Request::post("/user/login")
                                .auth()
                                .body(match state::get_string("anonymousId") {
                                    Ok(Some(anonymous_id)) => json!({ "anonymousId": anonymous_id }),
                                    _ => json!({}),
                                })
                                .send()
                                .await?;

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
            },
            Self::Logout => {
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
            },
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum UserSubcommand {
    #[clap(flatten)]
    Root(RootUserSubcommand),
    #[clap(subcommand)]
    Tokens(TokensSubcommand),
    Whoami {
        /// Output format to use
        #[clap(long, short, value_enum, value_parser, default_value_t)]
        format: OutputFormat,
        /// Only print the user's email address, this is quicker since it doesn't require a network
        /// request
        #[clap(long, short = 'e', value_parser)]
        only_email: bool,
    },
    Plan,
    ListAccounts {
        /// Output format to use
        #[clap(long, short, value_enum, value_parser, default_value_t)]
        format: OutputFormat,
    },
    Switch {
        /// Email to switch to
        #[clap(value_parser)]
        email: String,
    },
}

impl UserSubcommand {
    pub async fn execute(self) -> Result<()> {
        match self {
            Self::Root(cmd) => cmd.execute().await,
            Self::Tokens(cmd) => cmd.execute().await,
            Self::Whoami { format, only_email } => match fig_auth::get_email() {
                Some(email) => {
                    if only_email {
                        match format {
                            OutputFormat::Plain => println!("Email: {email}"),
                            OutputFormat::Json => println!("{}", json!({ "email": email })),
                            OutputFormat::JsonPretty => println!("{:#}", json!({ "email": email })),
                        }
                    } else {
                        let account = fig_api_client::user::account().await?;
                        match format {
                            OutputFormat::Plain => match account.username {
                                Some(username) => println!("Email: {}\nUsername: {}", account.email, username),
                                None => println!("Email: {}\nUsername is null", account.email),
                            },
                            OutputFormat::Json => println!("{}", serde_json::to_string(&account)?),
                            OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&account)?),
                        }
                    }
                    Ok(())
                },
                None => {
                    match format {
                        OutputFormat::Plain => println!("Not logged in"),
                        OutputFormat::Json => println!("{}", json!({ "email": null })),
                        OutputFormat::JsonPretty => println!("{:#}", json!({ "email": null })),
                    }
                    exit(1);
                },
            },
            Self::Plan => {
                println!("Plan: {:?}", fig_api_client::user::plans().await?.highest_plan());
                Ok(())
            },
            Self::ListAccounts { format } => {
                let files: Vec<String> = std::fs::read_dir(Credentials::account_credentials_dir()?)?
                    .filter_map(|file| file.ok())
                    .filter_map(|file| {
                        file.path()
                            .file_stem()
                            .and_then(|name| name.to_str())
                            .map(|name| name.into())
                    })
                    .collect();
                match format {
                    OutputFormat::Plain => {
                        for file in files {
                            println!("{file}");
                        }
                    },
                    OutputFormat::Json => println!("{}", json!(files)),
                    OutputFormat::JsonPretty => println!("{:#}", json!(files)),
                }
                Ok(())
            },
            Self::Switch { email } => {
                std::fs::copy(Credentials::account_credentials_path(email)?, Credentials::path()?)?;
                Ok(())
            },
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
