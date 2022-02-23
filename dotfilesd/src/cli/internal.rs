use std::{io::Write, process::exit};

use anyhow::{Context, Result};
use clap::Subcommand;
use crossterm::style::Stylize;
use serde_json::json;

use crate::util::settings;

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
        let source_immediately = settings::get_value("dotfiles.sourceImmediately")?
            .map(|s| s.as_str().map(|s| s.to_owned()))
            .flatten();

        match source_immediately.as_deref() {
            Some("never") => {}
            Some("always") => exit_code = 0,
            _ => {
                let mut stdout = std::io::stdout();

                stdout.write_all(
                    format!("{}", "Your dotfiles have been updated!\n".bold()).as_bytes(),
                )?;

                stdout.write_all(
                    format!(
                        "Would you like to update now? {} ",
                        "(y)es/(n)o/(A)lways/(N)ever".dim()
                    )
                    .as_bytes(),
                )?;

                stdout.flush()?;

                crossterm::terminal::enable_raw_mode()?;

                while let Ok(event) = crossterm::event::read() {
                    if let crossterm::event::Event::Key(key_event) = event {
                        match (key_event.code, key_event.modifiers) {
                            (crossterm::event::KeyCode::Char('y'), _) => {
                                crossterm::execute!(
                                    stdout,
                                    crossterm::cursor::MoveToNextLine(1),
                                    crossterm::style::Print(format!(
                                        "\n{}\n",
                                        "Updating dotfiles...".bold()
                                    )),
                                    crossterm::cursor::MoveToNextLine(1),
                                )?;

                                exit_code = 0;

                                break;
                            }
                            (crossterm::event::KeyCode::Char('n' | 'q'), _)
                            | (
                                crossterm::event::KeyCode::Char('c' | 'd'),
                                crossterm::event::KeyModifiers::CONTROL,
                            ) => {
                                crossterm::execute!(
                                    stdout,
                                    crossterm::cursor::MoveToNextLine(1),
                                    crossterm::style::Print(format!(
                                        "\n{}\n",
                                        "Skipping update...".bold()
                                    )),
                                    crossterm::cursor::MoveToNextLine(1),
                                )?;

                                break;
                            }
                            (crossterm::event::KeyCode::Char('A'), _) => {
                                crossterm::execute!(
                                    stdout,
                                    crossterm::cursor::MoveToNextLine(1),
                                    crossterm::style::Print(format!(
                                        "\n{}\n",
                                        "Always updating dotfiles...".bold()
                                    )),
                                    crossterm::cursor::MoveToNextLine(1),
                                )?;

                                exit_code = 0;

                                settings::set_value("dotfiles.sourceImmediately", json!("always"))?;

                                break;
                            }
                            (crossterm::event::KeyCode::Char('N'), _) => {
                                crossterm::execute!(
                                    stdout,
                                    crossterm::cursor::MoveToNextLine(1),
                                    crossterm::style::Print(format!(
                                        "\n{}\n",
                                        "Never updating dotfiles...".bold()
                                    )),
                                    crossterm::cursor::MoveToNextLine(1),
                                )?;

                                settings::set_value("dotfiles.sourceImmediately", json!("never"))?;

                                break;
                            }
                            _ => {}
                        }
                    }
                }

                stdout.flush()?;

                crossterm::terminal::disable_raw_mode()?;
            }
        };

        tokio::fs::write(&file, "").await?;
    }

    exit(exit_code);
}
