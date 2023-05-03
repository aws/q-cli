use std::io::Write;
use std::iter::empty;
use std::process::exit;
use std::time::Duration;

use arboard::Clipboard;
use clap::{
    ArgGroup,
    Subcommand,
};
use crossterm::style::Stylize;
use eyre::{
    bail,
    Result,
};
use fig_api_client::drip_campaign::DripCampaign;
use fig_api_client::user::Account;
use fig_ipc::local::logout_command;
use fig_ipc::{
    BufferedUnixStream,
    SendMessage,
    SendRecvMessage,
};
use fig_proto::secure::{
    clientbound,
    hostbound,
    Clientbound,
    Hostbound,
};
use fig_proto::secure_hooks::{
    new_account_info_request,
    new_confirm_exchange_credentials_request,
    new_start_exchange_credentials_request,
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
use fig_util::open_url;
use fig_util::system_info::is_remote;
use serde_json::{
    json,
    Value,
};
use time::format_description::well_known::Rfc3339;
use tracing::{
    error,
    info,
};

use super::OutputFormat;
use crate::util::spinner::SpinnerComponent;
use crate::util::{
    choose,
    dialoguer_theme,
    spinner,
};

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum RootUserSubcommand {
    /// Login to Fig
    #[command(group(
        // flag-action refers to the fact that it is a flag that is also an action :D
        ArgGroup::new("flag-actions")
            .multiple(false)
            .conflicts_with_all(["email", "switchable", "not_now"])
    ))]
    Login {
        /// Refresh the auth token if expired
        #[arg(long, short, group = "flag-actions")]
        refresh: bool,
        /// Force a refresh of the auth token
        #[arg(long, group = "flag-actions")]
        hard_refresh: bool,
        /// Email to login to
        #[arg(long, short)]
        email: Option<String>,
        /// Login with a fig user token
        #[arg(long, short, group = "flag-actions")]
        token: Option<String>,

        // Hidden flags
        /// Allow switching between accounts
        #[arg(long, hide = true)]
        switchable: bool,
        /// Add a "not now" option to the choicer
        #[arg(long, hide = true)]
        not_now: bool,
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
                not_now,
                token,
            } => {
                if refresh || hard_refresh {
                    let mut creds = Credentials::load_credentials()?;
                    if creds.is_expired() || hard_refresh {
                        creds.refresh_credentials().await?;
                        creds.save_credentials()?;
                    }
                    return Ok(());
                }

                if let Some(email) = fig_request::auth::get_email().await {
                    if not_now {
                        return Ok(());
                    } else {
                        eyre::bail!("Already logged in as {email}, please logout first.");
                    }
                }

                if let Some(token) = &token {
                    let mut spin = spinner::Spinner::new(vec![
                        SpinnerComponent::Text("Getting account info ".to_string()),
                        SpinnerComponent::Spinner,
                    ]);

                    let account: Account = fig_request::Request::get("/user/account")
                        .custom_token(token.clone())
                        .deser_json()
                        .await?;

                    Credentials::new_fig_token(Some(account.email.clone()), Some(token.clone())).save_credentials()?;

                    spin.stop_with_message(format!("Logged in as {}\n", account.email.magenta()));
                } else {
                    const OPTION_GITHUB: &str = "Sign in with GitHub";
                    const OPTION_EMAIL: &str = "Sign in with Email";
                    const OPTION_REMOTE: &str = "Sign in with local machine";
                    const OPTION_NOT_NOW: &str = "Not now";

                    let mut options = vec![];
                    if is_remote() {
                        let result = (|| async move {
                            let mut connection =
                            BufferedUnixStream::connect(fig_util::directories::secure_socket_path()?).await?;
                            let response: Option<Clientbound> = connection
                                .send_recv_message_filtered(
                                    Hostbound {
                                        packet: Some(hostbound::Packet::Request(new_account_info_request())),
                                    },
                                    |x: &Clientbound| matches!(x.packet, Some(clientbound::Packet::Response(_))),
                                )
                                .await?;
                            if let Some(response) = response {
                                let Some(clientbound::Packet::Response(response)) = response.packet else {
                                    unreachable!();
                                };
                                let Some(clientbound::response::Response::AccountInfo(account_info)) = response.response else {
                                    eyre::bail!("weird packet from desktop");
                                };
                                if account_info.logged_in {
                                    return Ok(true)
                                }
                            }
                            Ok(false)
                        })().await;
                        match result {
                            Ok(true) => options.push(OPTION_REMOTE),
                            Ok(false) => info!("local host not logged in"),
                            Err(err) => error!(%err, "failed checking local credentials"),
                        }
                    }
                    options.push(OPTION_GITHUB);
                    options.push(OPTION_EMAIL);
                    if not_now {
                        options.push(OPTION_NOT_NOW);
                    }

                    let chosen = match options.len() {
                        1 => options[0],
                        _ => {
                            options[choose(
                                if not_now {
                                    "Would you like to log in?"
                                } else {
                                    "Select action"
                                },
                                &options,
                            )?]
                        },
                    };

                    match chosen {
                        OPTION_NOT_NOW => {},
                        OPTION_GITHUB => {
                            // ! First copy your one-time code: 82AD-4E27
                            // Press Enter to open github.com in your browser...

                            let out = fig_request::Request::post("/auth/github/device-code").json().await?;

                            let device_code = out["deviceCode"].as_str().unwrap();
                            let user_code = out["userCode"].as_str().unwrap();
                            let verification_uri = out["verificationUri"].as_str().unwrap();
                            let _expires_in = out["expiresIn"].as_u64().unwrap();
                            let interval = out["interval"].as_u64().unwrap();

                            // Try to copy the code to the clipboard
                            if let Ok(mut clipboard) = Clipboard::new() {
                                clipboard.set_text(user_code).ok();
                            }

                            println!();
                            println!("First copy your one-time code: {}", user_code.bold().magenta());
                            print!(
                                "{} to open github.com in your browser... ",
                                "Press Enter".bold().magenta()
                            );
                            std::io::stdout().flush()?;

                            let _ = std::io::stdin().read_line(&mut String::new())?;

                            match open_url(verification_uri) {
                                Ok(_) => println!("Opened {} in your browser", verification_uri.bold().magenta()),
                                Err(_) => println!(
                                    "Failed to open browser, please open {} in your browser",
                                    verification_uri.bold().magenta()
                                ),
                            }

                            println!();

                            let mut spin = spinner::Spinner::new(vec![
                                SpinnerComponent::Text("Waiting for login ".to_string()),
                                SpinnerComponent::Spinner,
                            ]);

                            loop {
                                tokio::time::sleep(Duration::from_secs(interval)).await;

                                let res = fig_request::Request::post("/auth/github/device-poll")
                                    .body_json(json!({
                                        "deviceCode": device_code,
                                    }))
                                    .json()
                                    .await?;

                                match res["type"].as_str().unwrap() {
                                    "Pending" => continue,
                                    "SlowDown" => {
                                        tokio::time::sleep(Duration::from_secs(5)).await;
                                        continue;
                                    },
                                    "Success" => {
                                        let email = res["email"].as_str().unwrap();
                                        let access_token = res["accessToken"].as_str().unwrap();
                                        let id_token = res["idToken"].as_str().unwrap();
                                        let refresh_token = res["refreshToken"].as_str().unwrap();

                                        let creds = Credentials::new_jwt(
                                            Some(email.to_owned()),
                                            Some(access_token.to_owned()),
                                            Some(id_token.to_owned()),
                                            Some(refresh_token.to_owned()),
                                            false,
                                        );
                                        creds.save_credentials()?;

                                        spin.stop_with_message(format!("Logged in as {}\n", email.bold().magenta()));

                                        break;
                                    },
                                    other => eyre::bail!("Unexpected response from github: {other}"),
                                }
                            }
                        },
                        OPTION_EMAIL => {
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
                                        DripCampaign::load().await.ok();

                                        if switchable {
                                            let dir = Credentials::account_credentials_dir()?;
                                            if !dir.exists() {
                                                std::fs::create_dir_all(&dir)?;
                                            }
                                            std::fs::copy(
                                                Credentials::path()?,
                                                Credentials::account_credentials_path(trimmed_email)?,
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
                                                true,
                                            ),
                                            Request::post("/user/login").auth().body_json(login_body).send()
                                        );

                                        telem_join.ok();
                                        login_join?;

                                        println!();
                                        println!("Logged in as {}", trimmed_email.bold().magenta());
                                        println!();

                                        break;
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
                        OPTION_REMOTE => {
                            let mut connection =
                                BufferedUnixStream::connect(fig_util::directories::secure_socket_path()?).await?;
                            connection
                                .send_message(Hostbound {
                                    packet: Some(hostbound::Packet::Request(new_start_exchange_credentials_request())),
                                })
                                .await?;
                            let code = dialoguer::Input::with_theme(&dialoguer_theme())
                                .allow_empty(true)
                                .validate_with(|code: &String| match code.len() {
                                    0 => Ok(()),
                                    8 => {
                                        if code.chars().any(|x| !x.is_numeric()) {
                                            Err(eyre::eyre!("Codes should only have numbers"))
                                        } else {
                                            Ok(())
                                        }
                                    },
                                    _ => Err(eyre::eyre!("Codes should be 8 digits")),
                                })
                                .with_prompt("Enter your exchange code")
                                .interact_text()?;
                            if code.is_empty() {
                                eyre::bail!("Cancelled");
                            }
                            let response: Option<Clientbound> = connection
                                .send_recv_message_timeout_filtered(
                                    Hostbound {
                                        packet: Some(hostbound::Packet::Request(
                                            new_confirm_exchange_credentials_request(code),
                                        )),
                                    },
                                    Duration::from_secs(5),
                                    |x: &Clientbound| matches!(x.packet, Some(clientbound::Packet::Response(_))),
                                )
                                .await?;
                            if let Some(response) = response {
                                let Some(clientbound::Packet::Response(response)) = response.packet else {
                                    unreachable!();
                                };
                                let Some(clientbound::response::Response::ExchangeCredentials(exchange_credentials)) = response.response else {
                                    eyre::bail!("weird packet from desktop");
                                };
                                if exchange_credentials.approved {
                                    let creds: String = exchange_credentials.credentials.unwrap();
                                    tokio::fs::write(Credentials::path()?, creds).await?;

                                    let mut creds = Credentials::load_credentials()?;
                                    creds.refresh_credentials().await?;
                                    creds.save_credentials()?;
                                    DripCampaign::load().await.ok();
                                } else {
                                    eyre::bail!("Bad code");
                                }
                            }
                        },
                        _ => unreachable!(),
                    }
                }

                let mut spin = spinner::Spinner::new(vec![
                    SpinnerComponent::Text("Finishing up ".into()),
                    SpinnerComponent::Spinner,
                ]);

                if let Err(err) = fig_api_client::settings::sync().await {
                    error!(%err, "Failed to sync settings");
                }

                let daemon = fig_daemon::Daemon::default();

                let (dotfiles_res, plugins_res, script_res, daemon_res) = tokio::join!(
                    fig_sync::dotfiles::download_and_notify(false),
                    fig_sync::plugins::fetch_installed_plugins(false),
                    fig_api_client::scripts::sync_scripts(),
                    daemon.restart()
                );

                if let Err(err) = dotfiles_res {
                    error!(%err, "Failed to sync dotfiles");
                }

                if let Err(err) = plugins_res {
                    error!(%err, "Failed to sync plugins");
                }

                if let Err(err) = script_res {
                    error!(%err, "Failed to sync scripts");
                }

                if let Err(err) = daemon_res {
                    error!(%err, "Failed to restart daemon");
                }

                spin.stop_with_message("Done setting up Fig".into());

                // We assume that if this is a token login the user is already using the dashboard and we don't need
                // to direct them to it
                if token.is_none() {
                    println!();
                    println!("Run {} to get started", "fig".magenta());
                    println!();
                }

                Ok(())
            },
            Self::Logout => {
                let telem_join = tokio::spawn(fig_telemetry::dispatch_emit_track(
                    TrackEvent::new(
                        TrackEventType::Logout,
                        TrackSource::Cli,
                        env!("CARGO_PKG_VERSION").into(),
                        empty::<(&str, &str)>(),
                    ),
                    false,
                    true,
                ));

                let logout_join = logout_command();

                fig_request::auth::logout()?;

                let (_, _) = tokio::join!(telem_join, logout_join);

                println!("You are now logged out");
                println!("Run {} to log back in to Fig", "fig login".magenta());
                Ok(())
            },
        }
    }
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
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
            Self::Whoami { format, only_email } => match fig_request::auth::get_email().await {
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

#[derive(Debug, PartialEq, Eq, Subcommand)]
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
        team: Option<String>,
    },
    List {
        /// The team namespace to list the tokens for
        #[arg(long, short)]
        team: Option<String>,
        #[arg(long, short, value_enum, default_value_t)]
        format: OutputFormat,
    },
    Revoke {
        /// The name of the token to revoke
        name: String,
        /// The team namespace to revoke the token for
        #[arg(long, short)]
        team: Option<String>,
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
                    .body_json(json!({ "name": name, "team": team, "expiresAt": expires_at }))
                    .json()
                    .await?;

                match json.get("token").and_then(|x| x.as_str()) {
                    Some(val) => {
                        eprintln!("API token (make sure to save this as it will never be shown again):");
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
                    .body_json(json!({ "namespace": team }))
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
                let res = Request::post("/auth/tokens/revoke")
                    .auth()
                    .body_json(json!({ "team": team, "name": name }))
                    .json()
                    .await?;

                println!(
                    "Revoked token {} for team {}",
                    res["name"], res["namespace"]["username"]
                );

                Ok(())
            },
            Self::Validate { token } => {
                let valid = Request::post("/auth/tokens/validate")
                    .auth()
                    .body_json(json!({ "token": token }))
                    .json()
                    .await?;

                if let Some(Value::String(username)) = valid.get("username") {
                    println!("{username}");
                    Ok(())
                } else {
                    exit(1);
                }
            },
        }
    }
}
