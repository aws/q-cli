use clap::Args;
use crossterm::style::Stylize;
use dialoguer::Select;
use eyre::Result;
use fig_diagnostic::Diagnostics;
use owo_colors::{
    OwoColorize,
    Rgb,
};
use regex::Regex;
use supports_color::Stream;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

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
    pub async fn execute(&self) -> Result<()> {
        println!(
            "Please run {} and then share that output + your issue in the {} slack channel",
            "cw diagnostic".magenta(),
            "#codewhisperer-command-line-interest".bold()
        );
        println!();
        println!("This is temporary, we will have better issue management resources soon.");
        return Ok(());

        // Check if fig is running
        if !self.force && !fig_util::is_codewhisperer_desktop_running() {
            println!(
                "\n→ CodeWhisperer is not running.\n  Please launch CodeWhisperer with {} or run {} to create the issue anyways",
                "cw launch".magenta(),
                "cw issue --force".magenta()
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

        if let Some(issues) = search_github_issues(&issue_title).await {
            if !issues.is_empty() {
                let issue_count = 9;
                let mut options: Vec<String> = issues
                    .iter()
                    .take(issue_count)
                    .enumerate()
                    .map(|(i, (title, number, _, created_at))| {
                        let number = format!("#{number}");
                        let number = match supports_color::on(Stream::Stderr) {
                            Some(support) => match (support.has_basic, support.has_16m) {
                                (_, true) => {
                                    let start = Rgb(255, 0, 0);
                                    let end = Rgb(0, 0, 255);
                                    let r_step = (end.0 as isize - start.0 as isize) / (issue_count as isize - 1);
                                    let g_step = (end.1 as isize - start.1 as isize) / (issue_count as isize - 1);
                                    let b_step = (end.2 as isize - start.2 as isize) / (issue_count as isize - 1);

                                    number
                                        .color(Rgb(
                                            (start.0 as isize + i as isize * r_step) as u8,
                                            (start.1 as isize + i as isize * g_step) as u8,
                                            (start.2 as isize + i as isize * b_step) as u8,
                                        ))
                                        .to_string()
                                },
                                (true, _) => number.blue().to_string(),
                                _ => number,
                            },
                            None => number,
                        };

                        let len = 44;
                        let truncated_title = match title.len() > len {
                            true => format!("{}...", &title[0..len - 3].trim_end()),
                            false => title.clone(),
                        };

                        let padding_len = len - truncated_title.len();
                        let padding = " ".repeat(padding_len);

                        let created_at_date = OffsetDateTime::parse(created_at, &Rfc3339).unwrap();
                        let duration = OffsetDateTime::now_utc() - created_at_date;
                        let duration_str = match (
                            duration.whole_days(),
                            duration.whole_hours(),
                            duration.whole_minutes(),
                            duration.whole_seconds(),
                        ) {
                            (d, _, _, _) if d == 1 => format!("{} day ago", d),
                            (d, _, _, _) if d > 1 => format!("{} days ago", d),
                            (_, h, _, _) if h == 1 => format!("{} hour ago", h),
                            (_, h, _, _) if h > 1 => format!("{} hours ago", h),
                            (_, _, m, _) if m == 1 => format!("{} minute ago", m),
                            (_, _, m, _) if m > 1 => format!("{} minutes ago", m),
                            (_, _, _, s) if s == 1 => format!("{} second ago", s),
                            (_, _, _, s) => format!("{} seconds ago", s),
                        };

                        format!(
                            "{number}: {truncated_title}{padding} · {}",
                            duration_str.italic().color(Rgb(127, 127, 127))
                        )
                    })
                    .collect();
                options.push("Create new issue".bold().to_string());

                let selected = Select::with_theme(&crate::util::dialoguer_theme())
                    .with_prompt("Select an existing issue or create a new one")
                    .default(0)
                    .items(&options)
                    .interact()?;

                if selected < options.len() - 1 {
                    let (_, _, url, _) = &issues[selected];
                    println!("Opening issue in the browser...");
                    fig_util::open_url(url).expect("Failed to open issue in browser");
                    return Ok(());
                }
            }
        }

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

        let environment = Diagnostics::new();

        let os = match &environment.os {
            Some(os) => os.to_string(),
            None => "None".to_owned(),
        };

        let env_string = environment.user_readable().join("\n");

        let url = url::Url::parse_with_params("https://github.com/withfig/fig/issues/new", &[
            ("template", "1_main_issue_template.yml"),
            ("title", &issue_title),
            ("labels", &labels.join(",")),
            ("assignees", &assignees.join(",")),
            ("os", &os),
            ("environment", &env_string),
        ])?;

        println!("Heading over to GitHub...");
        if fig_util::open_url(url.as_str()).is_err() {
            println!("Issue Url: {}", url.as_str().underlined());
        }

        Ok(())
    }
}

async fn search_github_issues(query: &str) -> Option<Vec<(String, usize, String, String)>> {
    let client = fig_request::reqwest_client::reqwest_client(true)?;

    let search_url = format!(
        "https://api.github.com/search/issues?q={}+repo:withfig/fig+type:issue",
        query
    );

    let Ok(response) = client
        .get(&search_url)
        .header("User-Agent", "fig_issue_search")
        .send()
        .await
    else {
        return None;
    };

    let Ok(json_response) = response.json::<serde_json::Value>().await else {
        return None;
    };

    let items = json_response.get("items")?;

    Some(
        items
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|item| {
                let title = item.get("title")?.as_str()?.to_owned();
                let html_url = item.get("html_url")?.as_str()?.to_owned();
                let number = item.get("number")?.as_u64()? as usize;
                let created_at = item.get("created_at")?.as_str()?.to_owned();
                Some((title, number, html_url, created_at))
            })
            .collect(),
    )
}
