use std::{io::Write, process::exit};

use anyhow::{Context, Result};
use clap::Subcommand;
use crossterm::style::Stylize;
use serde_json::json;

#[derive(Debug, Subcommand)]
#[clap(hide = true, alias = "_")]
pub enum InternalSubcommand {
    PromptDotfilesChanged,
}

impl InternalSubcommand {
    pub async fn execute(self) -> Result<()> {
        match self {
            InternalSubcommand::PromptDotfilesChanged => prompt_dotfiles_changed().await?,
        }
        Ok(())
    }
}

pub async fn prompt_dotfiles_changed() -> Result<()> {
    let mut exit_code = 1;

    let session_id = std::env::var("TERM_SESSION_ID")?;
    let tempdir = std::env::temp_dir();

    let file = tempdir
        .join("fig")
        .join("dotfiles_updates")
        .join(session_id);

    let file_content = match tokio::fs::read_to_string(&file).await {
        Ok(content) => content,
        Err(_) => {
            tokio::fs::create_dir_all(&file.parent().context("Unable to get parent")?).await?;
            tokio::fs::write(&file, "").await?;
            exit(exit_code);
        }
    };

    if file_content.contains("true") {
        println!("{}", "Your dotfiles have been updated!".bold());

        let source_immediately = fig_settings::get_value("dotfiles.sourceImmediately")?
            .map(|s| s.as_str().map(|s| s.to_owned()))
            .flatten();

        let source_updates = match source_immediately.as_deref() {
            Some("never") => false,
            Some("always") => true,
            _ => {
                println!("Would you like Fig to re-source your dotfiles in open terminals on updates? (y)es,(n)o");
                let mut result = false;

                crossterm::terminal::enable_raw_mode()?;
                while let Ok(event) = crossterm::event::read() {
                    if let crossterm::event::Event::Key(key_event) = event {
                        match (key_event.code, key_event.modifiers) {
                            (crossterm::event::KeyCode::Char('y' | 'Y'), _) => {
                                fig_settings::set_value(
                                    "dotfiles.sourceImmediately",
                                    json!("always"),
                                )?;
                                result = true;
                                break;
                            }
                            (crossterm::event::KeyCode::Char('n' | 'q' | 'N'), _)
                            | (
                                crossterm::event::KeyCode::Char('c' | 'd'),
                                crossterm::event::KeyModifiers::CONTROL,
                            ) => {
                                fig_settings::set_value(
                                    "dotfiles.sourceImmediately",
                                    json!("never"),
                                )?;
                                result = false;
                                break;
                            }
                            _ => {}
                        }
                    }
                }
                crossterm::terminal::disable_raw_mode()?;
                result
            }
        };

        if source_updates {
            println!(
                "Automatically sourcing in this terminal. Run {} to disable auto-sourcing.",
                "fig settings dotfiles.sourceImmediately never".magenta()
            );
            // Set exit code to source changes.
            exit_code = 0;
        } else {
            println!(
                "Run {} to manually apply changes in this terminal. Or {} to always source updates.",
                "fig source".magenta(),
                "fig settings dotfiles.sourceImmediately always".magenta()
            );
        }

        tokio::fs::write(&file, "").await?;
    }

    exit(exit_code);
}
