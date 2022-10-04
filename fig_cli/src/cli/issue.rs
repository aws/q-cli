use clap::Args;
use crossterm::style::Stylize;
use eyre::Result;
use regex::Regex;

use crate::cli::diagnostics::{
    Diagnostic,
    Diagnostics,
};

#[derive(Debug, Args)]
pub struct IssueArgs {
    /// Force issue creation
    #[clap(long, short = 'f')]
    force: bool,
    /// Issue description
    description: Vec<String>,
}

impl IssueArgs {
    pub async fn execute(&self) -> Result<()> {
        // Check if fig is running
        #[cfg(target_os = "macos")]
        if !self.force && !crate::util::is_app_running() {
            println!(
                "\n→ Fig is not running.\n  Please launch Fig with {} or run {} to create the issue anyways",
                "fig launch".magenta(),
                "fig issue --force".magenta()
            );
            return Ok(());
        }

        let joined_description = self.description.join(" ").trim().to_owned();

        let issue_title = match joined_description.len() {
            0 => dialoguer::Input::with_theme(&crate::util::dialoguer_theme())
                .with_prompt("Issue Title")
                .interact_text()?,
            _ => joined_description,
        };

        let mut assignees = vec![];
        let mut labels = vec![
            "NEED_TO_LABEL".into(),
            "type:bug".into(),
            format!("os:{}", std::env::consts::OS),
        ];

        match std::env::consts::OS {
            "macos" => assignees.push("mschrage"),
            "linux" => assignees.push("grant0417"),
            "windows" => assignees.push("chaynabors"),
            _ => {},
        }

        if Regex::new(r"(?i)cli").unwrap().is_match(&issue_title) {
            assignees.push("grant0417");
            labels.push("codebase:cli".into());
        }

        if Regex::new(r"(?i)figterm").unwrap().is_match(&issue_title) {
            assignees.push("grant0417");
            labels.push("codebase:figterm".into());
        }

        if Regex::new(r"(?i)ssh").unwrap().is_match(&issue_title) {
            labels.push("integration:docker".into());
        }

        if Regex::new(r"(?i)docker").unwrap().is_match(&issue_title) {
            labels.push("integration:ssh".into());
        }

        let environment = Diagnostics::new().await?.user_readable()?.join("\n");

        println!();
        println!("{}", "> Environment".bold());
        println!("```");
        println!("{environment}");
        println!("```");
        println!();

        let url = url::Url::parse_with_params("https://github.com/withfig/fig/issues/new", &[
            ("template", "1_main_issue_template.yml"),
            ("title", &issue_title),
            ("labels", &labels.join(",")),
            ("assignees", &assignees.join(",")),
            (
                "issue_details",
                "<!-- Include a detailed description of the issue, and a screenshot/video if you can! -->\n\n",
            ),
            ("environment", &environment),
        ])?;

        if fig_util::open_url(url.as_str()).is_err() {
            println!("Issue Url: {}", url.as_str().underlined());
        }

        Ok(())
    }
}
