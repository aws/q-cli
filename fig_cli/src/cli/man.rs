use anyhow::Result;
use reqwest::Url;

pub fn man(args: &[String]) -> Result<()> {
    let url = Url::parse(&format!("https://fig.io/manual/{}", args.join("/")))?;
    if fig_util::open_url(url.as_str()).is_err() {
        println!("Unable to open man page: {}", url);
    }
    Ok(())
}
