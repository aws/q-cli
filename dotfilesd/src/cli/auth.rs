//! CLI auth

use crate::cli::util::dialoguer_theme;

use anyhow::Result;
use crossterm::style::Stylize;
use fig_auth::{
    get_client, Credentials, SignInConfirmError, SignInError, SignInInput, SignUpInput,
};

/// Login to the dotfiles server
pub async fn login_cli(refresh: bool) -> Result<()> {
    let client_id = "hkinciohdp1i7h0imdk63a4bv";
    let client = get_client("dotfiles-cli")?;

    if refresh {
        let mut creds = Credentials::load_credentials()?;
        creds.refresh_credentials(&client, client_id).await?;
        creds.save_credentials()?;
        return Ok(());
    }

    println!("{}", "Login to Fig dotfiles".bold().magenta());

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

// Logout from the dotfiles server
pub async fn logout_cli() -> Result<()> {
    let mut creds = Credentials::load_credentials()?;
    creds.clear_cridentials();
    creds.save_credentials()?;

    println!("Logged out");

    Ok(())
}

pub async fn user_info_cli() -> Result<()> {
    let creds = Credentials::load_credentials()?;

    match creds.get_email() {
        Some(email) => {
            if creds.get_access_token().is_some()
                && creds.get_id_token().is_some()
                && creds.get_refresh_token().is_some()
            {
                println!("Logged in as {}", email);
            } else {
                println!("Not logged in");
            }
        }
        None => println!("Not logged in"),
    }

    Ok(())
}
