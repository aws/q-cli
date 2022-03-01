use crate::cli::util::open_url;

use anyhow::Result;
use crossterm::style::Stylize;
use rand::prelude::*;

const TWEET_OPTIONS: &[&str] = &[
    "I've added autocomplete to my terminal using @fig!\n\n🛠🆕👉️",
    "Add VSCode-style autocomplete to your existing terminal. Move faster with @fig.\n\n🛠🆕👉️",
];

pub fn tweet_cli() -> Result<()> {
    println!();
    println!("→ Opening Twitter...");
    println!();

    let mut rng = rand::thread_rng();

    let url = url::Url::parse_with_params(
        "https://twitter.com/intent/tweet",
        [
            (
                "text",
                *TWEET_OPTIONS.choose(&mut rng).unwrap_or(&TWEET_OPTIONS[0]),
            ),
            ("url", "https://fig.io"),
            ("related", "fig"),
        ],
    )?;

    // Open the default browser to the homepage
    // let url = "https://twitter.com/intent/tweet?text=I%27ve%20added%20autocomplete%20to%20my%20terminal%20using%20@fig!%0a%0a%F0%9F%9B%A0%F0%9F%86%95%F0%9F%91%89%EF%B8%8F&url=https://fig.io";
    if open_url(url.as_str()).is_err() {
        println!("{}", url.as_str().underlined());
    }

    Ok(())
}
