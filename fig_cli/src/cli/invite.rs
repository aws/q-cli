use anyhow::Result;
use arboard::Clipboard;
use crossterm::style::Stylize;
use fig_auth::get_email;

pub async fn invite_cli() -> Result<()> {
    match get_email() {
        Some(email) => {
            let link = fig_request::Request::get(format!("/waitlist/get-referral-link-from-email/{email}"))
                .auth()
                .text()
                .await;

            match link {
                Ok(link) => {
                    println!();
                    println!("{}", "Thank you for sharing Fig.".bold());
                    println!();
                    println!("> {}", link.clone().bold().magenta());

                    if let Ok(mut clipboard) = Clipboard::new() {
                        if clipboard.set_text(link).is_ok() {
                            println!("  Your referral link has been copied to the clipboard.");
                        }
                    }

                    println!();
                },
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
                },
            }
        },
        None => {
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
        },
    }

    Ok(())
}
