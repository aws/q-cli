use std::fs;
use std::path::PathBuf;

use anyhow::{
    Context,
    Result,
};
use clap::Args;
use crossterm::style::{
    Color,
    Stylize,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;

// var BuiltinThemes []string = []string{"dark", "light", "system"}
const BUILT_IN_THEMES: [&str; 3] = ["dark", "light", "system"];
const DEFAULT_THEME: &str = "dark";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Author {
    name: Option<String>,
    twitter: Option<String>,
    github: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Theme {
    author: Option<Author>,
    version: Option<String>,
}

fn theme_folder() -> Option<PathBuf> {
    let new_theme_dir = fig_install::themes::themes_directory();
    if new_theme_dir.as_ref().map(|dir| dir.exists()).unwrap_or(false) {
        new_theme_dir
    } else {
        fig_directories::home_dir().map(|home| home.join(".fig").join("themes"))
    }
}

#[derive(Debug, Args)]
pub struct ThemeArgs {
    #[clap(long, action, conflicts_with_all = &["folder", "theme"])]
    list: bool,
    #[clap(long, action, conflicts_with_all = &["list", "theme"])]
    folder: bool,
    #[clap(value_parser, conflicts_with_all = &["list", "folder"])]
    theme: Option<String>,
}

impl ThemeArgs {
    pub async fn execute(&self) -> Result<()> {
        let theme_dir = theme_folder().context("Could not get theme directory")?;

        if self.folder {
            println!("{}", theme_dir.display());
            return Ok(());
        }

        if self.list {
            for theme_entry in std::fs::read_dir(&theme_dir)? {
                if let Ok(theme_file_name) = theme_entry.map(|s| s.file_name()) {
                    if let Some(theme) = theme_file_name.to_str() {
                        println!("{}", theme.trim_end_matches(".json"));
                    }
                }
            }
            return Ok(());
        }

        match &self.theme {
            Some(theme_str) => {
                let theme_str = theme_str.as_str();
                let theme_path = theme_dir.join(format!("{theme_str}.json"));
                match fs::read_to_string(theme_path) {
                    Ok(theme_file) => {
                        let theme: Theme = serde_json::from_str(&theme_file)?;
                        let result = fig_settings::settings::set_value("autocomplete.theme", json!(theme_str)).await;
                        let author = theme.author;

                        println!();

                        let mut theme_line = format!("› Switching to theme '{}'", theme_str.bold());
                        match author {
                            Some(Author { name, twitter, github }) => {
                                if let Some(name) = name {
                                    theme_line.push_str(&format!(" by {}", name.bold()));
                                }

                                println!("{}", theme_line);

                                if let Some(twitter) = twitter {
                                    println!("  🐦 {}", twitter.with(Color::Rgb { r: 29, g: 161, b: 242 }));
                                }

                                if let Some(github) = github {
                                    println!("  💻 {}", format!("github.com/{}", github).underlined());
                                }
                            },
                            None => {
                                println!("{}", theme_line);
                            },
                        }
                        println!();
                        result?;
                        Ok(())
                    },
                    Err(_) => {
                        if BUILT_IN_THEMES.contains(&theme_str) {
                            let result =
                                fig_settings::settings::set_value("autocomplete.theme", json!(theme_str)).await;
                            println!("› Switching to theme '{}'", theme_str.bold());
                            result?;
                            Ok(())
                        } else {
                            anyhow::bail!("'{theme_str}' does not exist in {}", theme_dir.display())
                        }
                    },
                }
            },
            None => {
                let theme =
                    fig_settings::settings::get_value("autocomplete.theme")?.unwrap_or_else(|| json!(DEFAULT_THEME));

                let theme_str = theme.as_str().map(String::from).unwrap_or_else(|| {
                    serde_json::to_string_pretty(&theme).unwrap_or_else(|_| DEFAULT_THEME.to_string())
                });

                println!("{}", theme_str);
                Ok(())
            },
        }
    }
}
