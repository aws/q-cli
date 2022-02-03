use super::util::open_url;

use anyhow::Result;
use crossterm::style::Stylize;

pub fn tweet_cli() -> Result<()> {
    println!();
    println!("→ Opening Twitter...");
    println!();

    // Open the default browser to the homepage
    let url = "https://twitter.com/intent/tweet?text=I%27ve%20added%20autocomplete%20to%20my%20terminal%20using%20@fig!%0a%0a%F0%9F%9B%A0%F0%9F%86%95%F0%9F%91%89%EF%B8%8F&url=https://fig.io";
    if open_url(url).is_err() {
        println!("{}", url.underlined());
    }

    Ok(())
}
