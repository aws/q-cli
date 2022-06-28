use anyhow::Result;
use clap::Args;
use reqwest::Url;

#[derive(Debug, Args)]
pub struct ManArgs {
    #[clap(value_parser)]
    command: Vec<String>,
}

impl ManArgs {
    pub fn execute(&self) -> Result<()> {
        let url = Url::parse(&format!("https://fig.io/manual/{}", self.command.join("/")))?;
        if fig_util::open_url(url.as_str()).is_err() {
            println!("Unable to open man page: {}", url);
        }
        Ok(())
    }
}
