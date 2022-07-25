use std::fmt::Write;

use anyhow::Result;
use clap::Args;
use crossterm::style::Stylize;
use fig_util::get_parent_process_exe;
use regex::Regex;

use crate::cli::diagnostics::{
    Diagnostic,
    Diagnostics,
};
use crate::util::{
    get_fig_version,
    is_app_running,
    OSVersion,
};

#[derive(Debug, Args)]
pub struct IssueArgs {
    /// Force issue creation
    #[clap(long, short = 'f', value_parser)]
    force: bool,
    /// Issue description
    #[clap(value_parser)]
    description: Vec<String>,
}

impl IssueArgs {
    pub async fn execute(&self) -> Result<()> {
        // Check if fig is running
        if !self.force && !is_app_running() {
            println!(
                "\n→ Fig is not running.\n  Please launch Fig with {} or run {} to create the issue anyways",
                "fig launch".magenta(),
                "fig issue --force".magenta()
            );
            return Ok(());
        }

        let issue_title = self.description.join(" ");
        let mut assignees = vec!["mschrage"];

        if Regex::new(r"(?i)cli").unwrap().is_match(&issue_title) {
            assignees.push("grant0417");
            assignees.push("sullivan-sean");
        }

        if Regex::new(r"(?i)figterm").unwrap().is_match(&issue_title) {
            assignees.push("grant0417");
            assignees.push("sullivan-sean");
        }

        let mut body = "### Details:\n|OS|Fig|Shell|\n|-|-|-|\n".to_owned();

        let os_version: String = OSVersion::new().map(|v| v.into()).unwrap_or_default();
        let fig_version = get_fig_version().map(|(version, _)| version).unwrap_or_default();
        let shell = get_parent_process_exe().unwrap_or_default();
        writeln!(body, "|{}|{}|{}|", &os_version, &fig_version, &shell.display()).ok();
        body.push_str("fig diagnostic\n\n");

        let diagnostic = Diagnostics::new().await?.user_readable()?.join("\n");
        body.push_str(&diagnostic);

        println!("{}", &body);

        println!("\n→ Opening GitHub...\n");

        let url = url::Url::parse_with_params("https://github.com/withfig/fig/issues/new", &[
            ("labels", "NEED_TO_LABEL"),
            ("assignees", &assignees.join(",")),
            ("template", "1_main_issue_template.yml"),
            (
                "issue_details",
                "<!-- Include a detailed description of the issue, and a screenshot/video if you can! -->\n\n",
            ),
            ("environment", &body),
            ("title", &issue_title),
        ])?;

        if fig_util::open_url(url.as_str()).is_err() {
            println!("{}", url.as_str().underlined());
        }

        Ok(())
    }
}
