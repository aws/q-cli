use anyhow::{Context, Result};
use crossterm::style::{Color, Stylize};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;

// var BuiltinThemes []string = []string{"dark", "light", "system"}
const BUILT_IN_THEMES: [&str; 3] = ["dark", "light", "system"];

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Author {
    name: String,
    twitter: String,
    github: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Theme {
    author: Option<Author>,
    version: String,
}

pub async fn theme_cli(theme_str: Option<String>) -> Result<()> {
    match theme_str {
        Some(theme_str) => {
            // set theme
            let path = format!(
                "{}/.fig/themes/{}.json",
                fig_directories::home_dir()
                    .context("Could not get home directory")?
                    .display(),
                theme_str
            );
            match fs::read_to_string(path) {
                Ok(theme_file) => {
                    let theme: Theme = serde_json::from_str(&theme_file)?;
                    let remote_result =
                        fig_settings::settings::set_value("autocomplete.theme", json!(theme_str))
                            .await?;
                    let author = theme.author;

                    println!();

                    let mut theme_line = format!("› Switching to theme '{}'", theme_str.bold());
                    match author {
                        Some(Author {
                            name,
                            twitter,
                            github,
                        }) => {
                            if !name.is_empty() {
                                theme_line.push_str(&format!(" by {}", name.bold()));
                            }
                            println!("{}", theme_line);

                            if !twitter.is_empty() {
                                let twitter = twitter.with(Color::Rgb {
                                    r: 29,
                                    g: 161,
                                    b: 242,
                                });
                                println!("  🐦 {}", twitter);
                            }
                            if !github.is_empty() {
                                let github = format!("github.com/{}", github);
                                println!("  💻 {}", github.underlined());
                            }
                        }
                        None => {
                            println!("{}", theme_line);
                        }
                    }
                    println!();
                    if remote_result.is_err() {
                        println!("Failed to sync new settings.");
                    }
                    Ok(())
                }
                Err(_) => {
                    if BUILT_IN_THEMES.contains(&theme_str.as_ref()) {
                        let remote_result = fig_settings::settings::set_value(
                            "autocomplete.theme",
                            json!(theme_str),
                        )
                        .await?;
                        println!("› Switching to theme '{}'", theme_str.bold());
                        if remote_result.is_err() {
                            println!("Failed to sync new settings.");
                        }
                        Ok(())
                    } else {
                        anyhow::bail!("'{}' does not exist in ~/.fig/themes/\n", theme_str)
                    }
                }
            }
        }
        None => {
            let theme = fig_settings::settings::get_value("autocomplete.theme")?
                .unwrap_or_else(|| json!("dark"));
            println!("{}", serde_json::to_string_pretty(&theme)?);
            Ok(())
        }
    }
}
