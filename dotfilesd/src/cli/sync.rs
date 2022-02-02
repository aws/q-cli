//! Sync of dotfiles

use std::{
    io::{stdout, Write},
    process::{exit, Command},
};

use anyhow::{Context, Result};
use crossterm::style::Stylize;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::try_join;

use crate::{
    auth::Credentials,
    util::{shell::Shell, Settings},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DotfilesSourceRequest {
    email: String,
}

async fn sync_file(shell: &Shell) -> Result<()> {
    // Get the access token from defaults
    let token = Command::new("defaults")
        .arg("read")
        .arg("com.mschrage.fig")
        .arg("access_token")
        .output()
        .with_context(|| "Could not read access_token")?;

    let email = Credentials::load_credentials()
        .map(|creds| creds.email)
        .or_else(|_| {
            let out = Command::new("defaults")
                .arg("read")
                .arg("com.mschrage.fig")
                .arg("userEmail")
                .output()?;

            let email = String::from_utf8(out.stdout)?;

            anyhow::Ok(Some(email))
        })?;

    // Constuct the request body
    let body = serde_json::to_string(&DotfilesSourceRequest {
        email: email.unwrap_or_else(|| "".to_string()),
    })?;

    let download = reqwest::Client::new()
        .get(shell.get_remote_source()?)
        .header(
            "Authorization",
            format!("Bearer {}", String::from_utf8_lossy(&token.stdout).trim()),
        )
        .body(body)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    // Create path to dotfiles
    let cache_file = shell
        .get_data_path()
        .context("Could not get cache file path")?;
    let cache_folder = cache_file.parent().unwrap();

    // Create cache folder if it doesn't exist
    if !cache_folder.exists() {
        std::fs::create_dir_all(cache_folder)?;
    }

    let mut dest_file = std::fs::File::create(cache_file)?;
    dest_file.write_all(download.as_bytes())?;

    Ok(())
}

pub async fn sync_all_files() -> Result<()> {
    try_join!(
        sync_file(&Shell::Bash),
        sync_file(&Shell::Zsh),
        sync_file(&Shell::Fish),
    )?;

    Ok(())
}

/// Download the lastest dotfiles
pub async fn sync_cli() -> Result<()> {
    sync_all_files().await?;

    println!("Dotfiles synced!");

    Ok(())
}

pub async fn prompt_cli() -> Result<()> {
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
        let settings = Settings::load()?;

        let enabled = settings
            .get_setting()
            .map(|map| map.get("dotfiles.prompt-update"))
            .map(|opt| opt.map(|value| value.as_bool()))
            .flatten()
            .flatten();

        match enabled {
            Some(false) => {}
            Some(true) => exit_code = 0,
            None => {
                let mut stdout = stdout();

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

                                let mut settings = Settings::load()?;
                                settings.get_mut_settings().map(|obj| {
                                    obj.insert(
                                        "dotfiles.prompt-update".to_string(),
                                        json!(true),
                                    )
                                });
                                settings.save()?;

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

                                let mut settings = Settings::load()?;
                                settings.get_mut_settings().map(|obj| {
                                    obj.insert(
                                        "dotfiles.prompt-update".to_string(),
                                        json!(false),
                                    )
                                });
                                settings.save()?;

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
