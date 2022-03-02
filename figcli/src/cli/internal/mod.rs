pub mod local_state;

use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::exit;

use crate::cli::installation::{self, InstallComponents};
use rand::distributions::{Alphanumeric, DistString};

use anyhow::{Context, Result};
use clap::{ArgGroup, Args, Subcommand};
use crossterm::style::Stylize;
use fig_ipc::hook::send_hook_to_socket;
use fig_proto::hooks::new_callback_hook;
use serde_json::json;

use native_dialog::{MessageDialog, MessageType};

use tracing::{debug, info, trace};

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

#[derive(Debug, Args)]
pub struct InstallArgs {
    /// Install only the daemon
    #[clap(long, conflicts_with = "dotfiles")]
    daemon: bool,
    /// Install only the shell integrations
    #[clap(long)]
    dotfiles: bool,
    /// Don't confirm automatic installation.
    #[clap(long)]
    no_confirm: bool,
    /// Force installation of fig
    #[clap(long)]
    force: bool,
}

#[derive(Debug, Subcommand)]
#[clap(hide = true, alias = "_")]
pub enum InternalSubcommand {
    PromptDotfilesChanged,
    LocalState(local_state::LocalStateArgs),
    Callback(CallbackArgs),
    /// Install fig cli
    Install(InstallArgs),
    /// Uninstall fig cli
    Uninstall {
        /// Uninstall only the daemon
        #[clap(long)]
        daemon: bool,
        /// Uninstall only the shell integrations
        #[clap(long)]
        dotfiles: bool,
        /// Uninstall only the binary
        #[clap(long)]
        binary: bool,
    },
    WarnUserWhenUninstallingIncorrectly,
}

pub fn install_cli_from_args(install_args: InstallArgs) -> Result<()> {
    let InstallArgs {
        daemon,
        dotfiles,
        no_confirm,
        force,
    } = install_args;
    let install_components = if daemon || dotfiles {
        let mut install_components = InstallComponents::empty();
        install_components.set(InstallComponents::DAEMON, daemon);
        install_components.set(InstallComponents::DOTFILES, dotfiles);
        install_components
    } else {
        InstallComponents::all()
    };

    installation::install_cli(install_components, no_confirm, force)
}

const BUFFER_SIZE: usize = 1024;

impl InternalSubcommand {
    pub async fn execute(self) -> Result<()> {
        match self {
            InternalSubcommand::Install(args) => install_cli_from_args(args)?,
            InternalSubcommand::Uninstall {
                daemon,
                dotfiles,
                binary,
            } => {
                let uninstall_components = if daemon || dotfiles || binary {
                    let mut uninstall_components = InstallComponents::empty();
                    uninstall_components.set(InstallComponents::DAEMON, daemon);
                    uninstall_components.set(InstallComponents::DOTFILES, dotfiles);
                    uninstall_components.set(InstallComponents::BINARY, binary);
                    uninstall_components
                } else {
                    InstallComponents::all()
                };

                installation::uninstall_cli(uninstall_components)?
            }
            InternalSubcommand::PromptDotfilesChanged => prompt_dotfiles_changed().await?,
            InternalSubcommand::LocalState(local_state) => local_state.execute().await?,
            InternalSubcommand::Callback(CallbackArgs {
                handler_id,
                filename,
                exit_code,
            }) => {
                trace!("handlerId: {}", handler_id);

                let (filename, exit_code) = match (filename, exit_code) {
                    (Some(filename), Some(exit_code)) => {
                        trace!(
                            "callback specified filepath ({}) and exitCode ({}) to output!",
                            filename,
                            exit_code
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
                        trace!("Created tmp file: {}", tmp_path.display());

                        loop {
                            let size = stdin.read(&mut buffer)?;
                            if size == 0 {
                                break;
                            }
                            tmp_file.write_all(&buffer[..size])?;
                            trace!(
                                "Read {} bytes\n{}",
                                size,
                                std::str::from_utf8(&buffer[..size])?
                            );
                        }

                        let filename: String =
                            tmp_path.to_str().context("invalid file path")?.into();
                        trace!("Done reading from stdin!");
                        (filename, -1)
                    }
                };
                let hook = new_callback_hook(&handler_id, &filename, exit_code);

                info!(
                    "Sending 'handlerId: {}, filename: {}, exitcode: {}' over unix socket!\n",
                    handler_id, filename, exit_code
                );

                match send_hook_to_socket(hook).await {
                    Ok(()) => {
                        debug!("Successfully sent hook");
                    }
                    Err(e) => {
                        debug!("Couldn't send hook {}", e);
                    }
                }
            }
            InternalSubcommand::WarnUserWhenUninstallingIncorrectly => {
                MessageDialog::new()
                    .set_type(MessageType::Warning)
                    .set_title("Trying to uninstall Fig?")
                    .set_text("Please run `fig uninstall` rather than moving the app to the Trash.")
                    .show_alert()
                    .unwrap();
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
