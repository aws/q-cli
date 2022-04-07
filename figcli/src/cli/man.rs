use anyhow::Result;
use reqwest::Url;

use super::util::open_url;

pub fn man(args: &[String]) -> Result<()> {
    let url = Url::parse(&format!("https://fig.io/manual/{}", args.join("/")))?;
    if open_url(url.as_str()).is_err() {
        println!("Unable to open man page: {}", url);
    }
    Ok(())
}
