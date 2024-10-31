use std::process::ExitCode;

use anstream::{
    eprintln,
    println,
};
use clap::Args;
use crossterm::style::Stylize;
use eyre::Result;
use fig_diagnostic::Diagnostics;
use fig_util::system_info::is_remote;
use fig_util::{
    CLI_BINARY_NAME,
    GITHUB_DISCUSSIONS_REPO_NAME,
    PRODUCT_NAME,
};

#[derive(Debug, Args, PartialEq, Eq)]
pub struct IssueArgs {
    /// Force issue creation
    #[arg(long, short = 'f')]
    force: bool,
    /// Issue description
    description: Vec<String>,
}

impl IssueArgs {
    #[allow(unreachable_code)]
    pub async fn execute(&self) -> Result<ExitCode> {
        // Check if fig is running
        if !(self.force || is_remote() || crate::util::desktop::desktop_app_running()) {
            println!(
                "\n→ {PRODUCT_NAME} is not running.\n  Please launch {PRODUCT_NAME} with {} or run {} to create the issue anyways",
                format!("{CLI_BINARY_NAME} launch").magenta(),
                format!("{CLI_BINARY_NAME} issue --force").magenta()
            );
            return Ok(ExitCode::FAILURE);
        }

        let joined_description = self.description.join(" ").trim().to_owned();

        let issue_title = match joined_description.len() {
            0 => dialoguer::Input::with_theme(&crate::util::dialoguer_theme())
                .with_prompt("Issue Title")
                .interact_text()?,
            _ => joined_description,
        };

        let diagnostics = Diagnostics::new().await;

        let os = match &diagnostics.system_info.os {
            Some(os) => os.to_string(),
            None => "None".to_owned(),
        };

        let environment = match diagnostics.user_readable() {
            Ok(diagnostics) => diagnostics,
            Err(err) => {
                eprintln!("Error getting diagnostics: {err}");
                "Error occurred while generating diagnostics".to_owned()
            },
        };

        let url = url::Url::parse_with_params(
            &format!("https://github.com/aws/{GITHUB_DISCUSSIONS_REPO_NAME}/discussions/new"),
            &[
                ("category", "support-ticket"),
                ("title", &issue_title),
                ("os", &os),
                ("environment", &environment),
            ],
        )?;

        println!("Heading over to GitHub...");
        if is_remote() || fig_util::open_url_async(url.as_str()).await.is_err() {
            println!("Issue Url: {}", url.as_str().underlined());
        }

        Ok(ExitCode::SUCCESS)
    }
}
