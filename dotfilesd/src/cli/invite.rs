use base64::encode;
use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;
use crossterm::style::Stylize;
use std::process::Command;

use anyhow::{Context, Error, Result};
use serde_json::json;

use crate::auth::Credentials;

fn get_token() -> Result<String, Error> {
    Credentials::load_credentials()
        .map(|creds| {
            encode(
                json!({
                    "accessToken": creds.access_token,
                    "idToken": creds.id_token
                })
                .to_string(),
            )
        })
        .or_else(|_| {
            let token = Command::new("defaults")
                .arg("read")
                .arg("com.mschrage.fig")
                .arg("access_token")
                .output()
                .with_context(|| "Could not read access_token")?;

            Ok(String::from_utf8_lossy(&token.stdout).trim().to_string())
        })
}

fn get_email() -> Option<String> {
    Credentials::load_credentials()
        .map(|creds| creds.email)
        .or_else(|_| {
            let out = Command::new("defaults")
                .arg("read")
                .arg("com.mschrage.fig")
                .arg("userEmail")
                .output()?;

            let email = String::from_utf8(out.stdout)?.trim().to_string();

            anyhow::Ok(Some(email))
        })
        .ok()?
}

pub async fn invite_cli() -> Result<()> {
    let email = get_email();
    if let Some(email) = email {
        let response = reqwest::Client::new()
            .get(format!(
                "https://api.fig.io/waitlist/get-referral-link-from-email/{}",
                email
            ))
            .header("Authorization", format!("Bearer {}", get_token()?))
            .send()
            .await?
            .error_for_status();

        match response {
            Ok(response) => {
                let link = response.text().await?;

                let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                ctx.set_contents(link.to_owned()).unwrap();

                println!();
                println!("{}", "Thank you for sharing Fig.".bold());
                println!();
                println!("> {}", link.bold().magenta());
                println!("  Your referral link has been copied to the clipboard.");
                println!();
            }
            Err(_) => {
                println!();
                println!(
                    "{}{}{}",
                    "Error".bold().red(),
                    ": We can't find a referral code for this email address: ".bold(),
                    email.bold()
                );
                println!();
                println!(
                    "If you think there is a mistake, please contact {}",
                    "hello@fig.io".underlined()
                );
                println!();
            }
        }
    } else {
        println!();
        println!(
            "{}{}",
            "Error".bold().red(),
            ": It does not seem like you are logged into Fig.".bold()
        );
        println!();
        println!(
            "Run {} and follow the prompts to log back in. Then try again.",
            "fig user logout".bold().magenta()
        );
        println!();
    }

    Ok(())
}
