pub mod local_state;

use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::exit;

use rand::distributions::{Alphanumeric, DistString};

use anyhow::{Context, Result};
use clap::{ArgGroup, Args, Subcommand};
use crossterm::style::Stylize;
use fig_ipc::hook::send_hook_to_socket;
use fig_proto::hooks::new_callback_hook;
use serde_json::json;

#[derive(Debug, Args)]
#[clap(group(
        ArgGroup::new("output")
            .args(&["filename", "exit-code"])
            .multiple(true)
            .requires_all(&["filename", "exit-code"])
            ))]
pub struct CallbackArgs {
    handler_id: String,
    #[clap(group = "output")]
    filename: Option<String>,
    #[clap(group = "output")]
    exit_code: Option<i64>,
}

#[derive(Debug, Subcommand)]
#[clap(hide = true, alias = "_")]
pub enum InternalSubcommand {
    PromptDotfilesChanged,
    LocalState(local_state::LocalStateArgs),
    Callback(CallbackArgs),
}

const BUFFER_SIZE: usize = 1024;

impl InternalSubcommand {
    pub async fn execute(self) -> Result<()> {
        match self {
            InternalSubcommand::PromptDotfilesChanged => prompt_dotfiles_changed().await?,
            InternalSubcommand::LocalState(local_state) => local_state.execute().await?,
            InternalSubcommand::Callback(CallbackArgs {
                handler_id,
                filename,
                exit_code,
            }) => {
                println!("handlerId: {}", handler_id);

                let (filename, exit_code) = match (filename, exit_code) {
                    (Some(filename), Some(exit_code)) => {
                        println!(
                            "callback specified filepath ({}) and exitCode ({}) to output!",
                            filename, exit_code
                        );
                        (filename, exit_code)
                    }
                    _ => {
                        let file_id = Alphanumeric.sample_string(&mut rand::thread_rng(), 9);
                        let tmp_filename = format!("fig-callback-{}", file_id);
                        let tmp_path = PathBuf::from("/tmp").join(&tmp_filename);
                        let mut tmp_file = std::fs::File::create(&tmp_path)?;
                        let mut buffer = [0u8; BUFFER_SIZE];
                        let mut stdin = std::io::stdin();
                        println!("Created tmp file: {}", tmp_path.display());

                        loop {
                            let size = stdin.read(&mut buffer)?;
                            if size == 0 {
                                break;
                            }
                            tmp_file.write_all(&buffer[..size])?;
                            println!(
                                "Read {} bytes\n{}",
                                size,
                                std::str::from_utf8(&buffer[..size])?
                            );
                        }

                        let filename: String =
                            tmp_path.to_str().context("invalid file path")?.into();
                        println!("Done reading from stdin!");
                        (filename, -1)
                    }
                };
                let hook = new_callback_hook(&handler_id, &filename, exit_code);

                println!(
                    "Sending 'handlerId: {}, filename: {}, exitcode: {}' over unix socket!\n",
                    handler_id, filename, exit_code
                );

                match send_hook_to_socket(hook).await {
                    Ok(()) => {
                        println!("Successfully sent hook");
                    }
                    Err(e) => {
                        println!("Couldn't send hook {}", e);
                    }
                }
            }
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

        let source_immediately = fig_settings::settings::get_value("dotfiles.sourceImmediately")?
            .and_then(|s| s.as_str().map(|s| s.to_owned()));

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
                                fig_settings::settings::set_value(
                                    "dotfiles.sourceImmediately",
                                    json!("always"),
                                )
                                .await?
                                .ok();
                                result = true;
                                break;
                            }
                            (crossterm::event::KeyCode::Char('n' | 'q' | 'N'), _)
                            | (
                                crossterm::event::KeyCode::Char('c' | 'd'),
                                crossterm::event::KeyModifiers::CONTROL,
                            ) => {
                                fig_settings::settings::set_value(
                                    "dotfiles.sourceImmediately",
                                    json!("never"),
                                )
                                .await?
                                .ok();
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
