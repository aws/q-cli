use crate::{
    cli::{
        diagnostics::{Diagnostic, Diagnostics},
        util::{get_fig_version, open_url, OSVersion},
    },
    util::get_shell,
};

use anyhow::Result;
use crossterm::style::Stylize;
use regex::Regex;
use url::form_urlencoded;

pub async fn issue_cli(force: bool, description: Vec<String>) -> Result<()> {
    // Check if fig is running
    #[cfg(target_os = "macos")]
    {
        if !force && !crate::util::is_app_running() {
            println!("\n→ Fig is not running.\n  Please launch Fig with {} or run {} to create the issue anyways", "fig launch".magenta(), "fig issue --force".magenta());
            return Ok(());
        }
    }

    let text = description.join(" ");
    let mut assignees = vec!["mschrage"];

    if Regex::new(r"(?i)cli").unwrap().is_match(&text) {
        assignees.push("grant0417");
        assignees.push("sullivan-sean");
    }

    if Regex::new(r"(?i)figterm").unwrap().is_match(&text) {
        assignees.push("grant0417");
        assignees.push("sullivan-sean");
    }

    let mut body = "### Description:\n> Please include a detailed description of the issue (and an image or screen recording, if applicable)\n\n".to_owned();
    if !text.is_empty() {
        body.push_str(&text);
    }
    body.push_str("\n\n### Details:\n|OS|Fig|Shell|\n|-|-|-|\n");

    let os_version: String = OSVersion::new().map(|v| v.into()).unwrap_or_default();
    let fig_version = get_fig_version()
        .map(|(version, _)| version)
        .unwrap_or_default();
    let shell = get_shell().unwrap_or_default();
    body.push_str(&format!("|{}|{}|{}|\n", &os_version, &fig_version, &shell));
    body.push_str("<details><summary>Fig Diagnostic</summary>\n<p>\n\n");

    let diagnostic = Diagnostics::new().await?.user_readable()?.join("\n\n");
    body.push_str(&diagnostic);
    body.push_str("\n\n</p>\n</details>");

    println!("{}", &body);

    println!("\n→ Opening GitHub...\n");

    let params = form_urlencoded::Serializer::new(String::new())
        .append_pair("assignees", &assignees.join(","))
        .append_pair("body", &body)
        .finish();

    let url = format!("https://github.com/withfig/fig/issues/new?{}", params);
    if open_url(&url).is_err() {
        println!("{}", url.underlined());
    }

    Ok(())
}
