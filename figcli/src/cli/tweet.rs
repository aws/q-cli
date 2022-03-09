use crate::cli::util::open_url;

use anyhow::Result;
use crossterm::style::Stylize;
use rand::prelude::*;
use reqwest::Url;

const TWEET_OPTIONS: &[(&str, bool)] = &[
    ("I've added autocomplete to my terminal using @fig!\n\n🛠🆕👉️", true),
    ("I've added autocomplete to my terminal using @fig! It's super fast and integrates with my existing terminal.\n\n🛠🆕👉️", true),
    ("I just added autocomplete to my terminal using @fig! It supports 300+ CLI tools and fits into my workflow seamlessly!\n\n🛠🆕👉️", true),
    ("I just added IDE-style to my terminal using @fig. It supports 300+ CLI tools and works with my existing terminal! Try it out\n\n🛠🆕🔥", false),
];

pub fn tweet_cli() -> Result<()> {
    println!();
    println!("→ Opening Twitter...");
    println!();

    let mut rng = rand::thread_rng();
    let (tweet, with_link) = TWEET_OPTIONS.choose(&mut rng).unwrap_or(&TWEET_OPTIONS[0]);

    let mut params = vec![("text", *tweet), ("related", "fig")];

    if *with_link {
        params.push(("url", "https://fig.io"));
    }

    let url = Url::parse_with_params("https://twitter.com/intent/tweet", &params)?;

    // Open the default browser to the homepage
    // let url = "https://twitter.com/intent/tweet?text=I%27ve%20added%20autocomplete%20to%20my%20terminal%20using%20@fig!%0a%0a%F0%9F%9B%A0%F0%9F%86%95%F0%9F%91%89%EF%B8%8F&url=https://fig.io";
    if open_url(url.as_str()).is_err() {
        println!("{}", url.as_str().underlined());
    }

    Ok(())
}
