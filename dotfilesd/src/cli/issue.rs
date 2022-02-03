use super::diagnostics::summary;
use super::util::open_url;
use super::util::{get_fig_version, get_os_version};
use anyhow::{Context, Result};
use crossterm::style::Stylize;
use regex::Regex;
use std::process::Command;
use url::form_urlencoded;

pub fn get_shell() -> Result<String> {
    let ppid = nix::unistd::getppid();
    let result = Command::new("ps")
        .arg("-p")
        .arg(ppid.to_string())
        .arg("-o")
        .arg("comm=")
        .output()
        .with_context(|| "Could not read value")?;

    Ok(String::from_utf8_lossy(&result.stdout).trim().to_string())
}

pub async fn issue_cli(description: Vec<String>) -> Result<()> {
    let text = description.join(" ");
    let mut assignees = vec!["mschrage"];

    if Regex::new(r"(?i)cli").unwrap().is_match(&text) {
        assignees.push("grant0417");
    }

    if Regex::new(r"(?i)figterm").unwrap().is_match(&text) {
        assignees.push("sullivan-sean");
    }

    let mut body = "### Description:\n> Please include a detailed description of the issue (and an image or screen recording, if applicable)\n\n".to_owned();
    if !text.is_empty() {
        body.push_str(&text);
    }
    body.push_str("\n\n### Details:\n|OS|Fig|Shell|\n|-|-|-|\n");

    let os_version = get_os_version()
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "".to_owned());
    let fig_version = get_fig_version()
        .map(|(version, _)| version)
        .unwrap_or_else(|_| "".to_owned());
    let shell = get_shell().unwrap_or_else(|_| "".to_owned());
    body.push_str(&format!("|{}|{}|{}|\n", &os_version, &fig_version, &shell));
    body.push_str("<details><summary><code>fig diagnostic</code></summary>\n<p>\n<pre>");

    let diagnostic = summary().await?;
    body.push_str(&diagnostic);
    body.push_str("</pre>\n</p>\n</details>");

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
