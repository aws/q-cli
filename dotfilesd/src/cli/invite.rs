use anyhow::Result;
use copypasta::{ClipboardContext, ClipboardProvider};
use crossterm::style::Stylize;

use crate::util::auth::{get_email, get_token};

pub async fn invite_cli() -> Result<()> {
    let email = get_email();
    if let Some(email) = email {
        let token = get_token().await?;

        let response = reqwest::Client::new()
            .get(format!(
                "https://api.fig.io/waitlist/get-referral-link-from-email/{}",
                email
            ))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?
            .error_for_status();

        match response {
            Ok(response) => {
                let link = response.text().await?;

                println!();
                println!("{}", "Thank you for sharing Fig.".bold());
                println!();
                println!("> {}", link.clone().bold().magenta());

                if let Ok(mut ctx) = ClipboardContext::new() {
                    if ctx.set_contents(link).is_ok() {
                        println!("  Your referral link has been copied to the clipboard.");
                    }
                }

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
