use std::iter::empty;
use std::process::exit;

use clap::Subcommand;
use crossterm::style::Stylize;
use eyre::{
    bail,
    Result,
};
use fig_request::auth::{
    Credentials,
    SignInConfirmError,
    SignInInput,
};
use fig_request::Request;
use fig_settings::state;
use fig_telemetry::{
    TrackEvent,
    TrackEventType,
    TrackSource,
};
use serde_json::{
    json,
    Value,
};
use time::format_description::well_known::Rfc3339;

use super::OutputFormat;

#[derive(Subcommand, Debug)]
pub enum RootUserSubcommand {
    /// Login to Fig
    Login {
        /// Refresh the auth token if expired
        #[arg(long, short)]
        refresh: bool,
        /// Force a refresh of the auth token
        #[arg(long)]
        hard_refresh: bool,
        /// Email to login to
        email: Option<String>,
        ///
        #[arg(long, hide = true)]
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
                if refresh || hard_refresh {
                    let mut creds = Credentials::load_credentials()?;
                    if creds.is_expired() || hard_refresh {
                        creds.refresh_credentials().await?;
                        creds.save_credentials()?;
                    }
                    return Ok(());
                }

                if email.is_none() {
                    println!("{}", "Login to Fig".bold().magenta());
                }

                let email: String = match email {
                    Some(email) => email,
                    None => dialoguer::Input::with_theme(&crate::util::dialoguer_theme())
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
                let sign_in_input = SignInInput::new(trimmed_email);

                println!("Sending login code to {trimmed_email}...");
                println!("Please check your email for the code");

                let mut sign_in_output = sign_in_input.sign_in().await?;

                loop {
                    let login_code: String = dialoguer::Input::with_theme(&crate::util::dialoguer_theme())
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

                            let mut login_body = serde_json::Map::new();
                            login_body.insert("loginSource".into(), "cli".into());
                            if let Ok(Some(anonymous_id)) = state::get_string("anonymousId") {
                                login_body.insert("anonymousId".into(), anonymous_id.into());
                            };

                            let (telem_join, login_join) = tokio::join!(
                                fig_telemetry::dispatch_emit_track(
                                    TrackEvent::new(
                                        TrackEventType::Login,
                                        TrackSource::Cli,
                                        env!("CARGO_PKG_VERSION").into(),
                                        empty::<(&str, &str)>()
                                    ),
                                    false,
                                ),
                                Request::post("/user/login").auth().body(login_body).send()
                            );

                            telem_join.ok();
                            login_join?;

                            println!("Login successful!");
                            return Ok(());
                        },
                        Err(err) => match err {
                            SignInConfirmError::InvalidCode => {
                                println!("Code mismatch, try again...");
                                continue;
                            },
                            SignInConfirmError::TooManyAttempts => {
                                return Err(eyre::eyre!(
                                    "Not authorized, you may have entered the wrong code too many times."
                                ));
                            },
                            err => return Err(err.into()),
                        },
                    };
                }
            },
            Self::Logout => {
                fig_telemetry::dispatch_emit_track(
                    TrackEvent::new(
                        TrackEventType::Logout,
                        TrackSource::Cli,
                        env!("CARGO_PKG_VERSION").into(),
                        empty::<(&str, &str)>(),
                    ),
                    false,
                )
                .await
                .ok();

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
    #[command(flatten)]
    Root(RootUserSubcommand),
    /// Subcommand for dealing with tokens
    #[command(subcommand)]
    Tokens(TokensSubcommand),
    /// Prints details about the current user
    Whoami {
        /// Output format to use
        #[arg(long, short, value_enum, default_value_t)]
        format: OutputFormat,
        /// Only print the user's email address, this is quicker since it doesn't require a network
        /// request
        #[arg(long, short = 'e')]
        only_email: bool,
    },
    /// Prints details about the user's plan
    #[command(hide = true)]
    Plan {
        /// Output format to use
        #[arg(long, short, value_enum, default_value_t)]
        format: OutputFormat,
    },
    /// List all accounts that can be switch to
    #[command(hide = true)]
    ListAccounts {
        /// Output format to use
        #[arg(long, short, value_enum, default_value_t)]
        format: OutputFormat,
    },
    /// Switch to a switchable account
    #[command(hide = true)]
    Switch {
        /// Email to switch to
        email: String,
    },
}

impl UserSubcommand {
    pub async fn execute(self) -> Result<()> {
        match self {
            Self::Root(cmd) => cmd.execute().await,
            Self::Tokens(cmd) => cmd.execute().await,
            Self::Whoami { format, only_email } => match fig_request::auth::get_email() {
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
            Self::Plan { format } => {
                let plan = fig_api_client::user::plans().await?;
                match format {
                    OutputFormat::Plain => println!("Plan: {:?}", plan.highest_plan()),
                    OutputFormat::Json => println!("{}", serde_json::to_string(&plan)?),
                    OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&plan)?),
                }
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
        name: String,
        /// The expiration date of the token in RFC3339 format
        #[arg(long, conflicts_with = "expires_in")]
        expires_date: Option<String>,
        /// The time till the token expires (e.g. "90d")
        #[arg(long, conflicts_with = "expires_date")]
        expires_in: Option<String>,
        /// The team namespace to create the token for
        #[arg(long, short)]
        team: String,
    },
    List {
        /// The team namespace to list the tokens for
        #[arg(long, short)]
        team: String,
        #[arg(long, short, value_enum, default_value_t)]
        format: OutputFormat,
    },
    Revoke {
        /// The name of the token to revoke
        name: String,
        /// The team namespace to revoke the token for
        #[arg(long, short)]
        team: String,
    },
    /// Validate a token is valid
    Validate {
        /// The token to validate
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
