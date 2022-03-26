pub mod local_state;

use crate::cli::installation::{self, InstallComponents};
use crate::dotfiles::notify::TerminalNotification;

use anyhow::{Context, Result};
use clap::{ArgGroup, Args, Subcommand};
use crossterm::style::Stylize;
use fig_ipc::hook::send_hook_to_socket;
use fig_proto::hooks::new_callback_hook;
use native_dialog::{MessageDialog, MessageType};
use rand::distributions::{Alphanumeric, DistString};
use std::{
    io::{Read, Write},
    path::PathBuf,
    process::exit,
    str::FromStr,
};
use tracing::{debug, error, info, trace};

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
    /// Prompt the user that the dotfiles have changes
    /// Also use for `fig source` internals
    PromptDotfilesChanged,
    /// Change the local-state file
    LocalState(local_state::LocalStateArgs),
    /// Callback used for the internal psudoterminal
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
    /// Notify the user that they are uninstalling incorrectly
    WarnUserWhenUninstallingIncorrectly,
    GetShell,
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
            InternalSubcommand::GetShell => {
                #[cfg(unix)]
                {
                    let pid = nix::unistd::getppid();
                    let mut buff = vec![0; 1024];

                    #[cfg(target_os = "macos")]
                    let out_buf = {
                        use nix::libc::proc_pidpath;

                        // TODO: Make sure pid exists or that access is allowed?
                        let ret = unsafe {
                            nix::libc::proc_pidpath(
                                pid.as_raw(),
                                buff.as_mut_ptr() as *mut std::ffi::c_void,
                                buff.len() as u32,
                            )
                        };

                        if ret == 0 {
                            exit(1);
                        }

                        &buff[..ret as usize]
                    };

                    #[cfg(target_os = "linux")]
                    let out_buf = {
                        loop {
                            let ret = unsafe {
                                nix::libc::readlink(
                                    format!("/proc/{}/exe", pid).as_str().as_ptr(),
                                    buff.as_mut_ptr() as *mut std::ffi::c_void,
                                    buff.len() as u32,
                                )
                            };

                            if ret == -1 {
                                exit(1);
                            }

                            if ret == buff.len() as i32 {
                                buff.resize(buff.len() * 2, 0);
                                continue;
                            }

                            break &buff[..ret as usize];
                        }
                    };

                    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
                    {
                        exit(1);
                    }

                    match std::str::from_utf8(out_buf) {
                        Ok(path) => print!("{}", path),
                        Err(_) => exit(1),
                    }
                }

                #[cfg(windows)]
                {
                    return Err(anyhow!("This is unimplemented on Windows"));
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum UpdatedVerbosity {
    None,
    Minimal,
    Full,
}

pub async fn prompt_dotfiles_changed() -> Result<()> {
    // An exit code of 0 will source the new changes
    // An exit code of 1 will not source the new changes

    let session_id = match std::env::var("TERM_SESSION_ID") {
        Ok(session_id) => session_id,
        Err(err) => {
            error!("Couldn't get TERM_SESSION_ID: {}", err);
            exit(1);
        }
    };

    let file = std::env::temp_dir()
        .join("fig")
        .join("dotfiles_updates")
        .join(session_id);

    let file_clone = file.clone();
    ctrlc::set_handler(move || {
        crossterm::execute!(std::io::stdout(), crossterm::cursor::Show,).ok();
        std::fs::write(&file_clone, "").ok();

        exit(1);
    })
    .ok();

    let file_content = match tokio::fs::read_to_string(&file).await {
        Ok(content) => content,
        Err(_) => {
            if let Err(err) =
                tokio::fs::create_dir_all(&file.parent().expect("Unable to create parent dir"))
                    .await
            {
                error!("Unable to create directory: {}", err);
            }

            if let Err(err) = tokio::fs::write(&file, "").await {
                error!("Unable to write to file: {}", err);
            }

            exit(1);
        }
    };

    let exit_code = match TerminalNotification::from_str(&file_content) {
        Ok(TerminalNotification::Source) => {
            println!();
            println!("{}", "✅ Dotfiles sourced!".bold());
            println!();

            0
        }
        Ok(TerminalNotification::NewUpdates) => {
            let verbosity = match fig_settings::settings::get_value("dotfiles.verbosity")
                .ok()
                .flatten()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .as_deref()
            {
                Some("none") => UpdatedVerbosity::None,
                Some("minimal") => UpdatedVerbosity::Minimal,
                Some("full") => UpdatedVerbosity::Full,
                _ => UpdatedVerbosity::Minimal,
            };

            let source_immediately =
                fig_settings::settings::get_value("dotfiles.sourceImmediately")
                    .ok()
                    .flatten()
                    .and_then(|s| s.as_str().map(|s| s.to_owned()));

            let source_updates = match source_immediately.as_deref() {
                Some("always") => true,
                // Ask is depercated
                // Some("ask") => {
                //     let dialog_result =  dialoguer::Select::with_theme(&dialoguer_theme())
                //             .with_prompt("In the future, would you like Fig to auto-apply dotfiles changes in open terminals?")
                //             .items(&["Yes", "No"])
                //             .default(0)
                //             .interact_opt();

                //     match dialog_result {
                //         Ok(Some(0)) => {
                //             fig_settings::settings::set_value(
                //                 "dotfiles.sourceImmediately",
                //                 json!("always"),
                //             )
                //             .await
                //             .ok();

                //             true
                //         }
                //         Ok(Some(1)) => {
                //             fig_settings::settings::set_value(
                //                 "dotfiles.sourceImmediately",
                //                 json!("never"),
                //             )
                //             .await
                //             .ok();

                //             false
                //         }
                //         _ => false,
                //     }
                // }
                Some("never") => false,
                _ => false,
            };

            if source_updates {
                if verbosity >= UpdatedVerbosity::Minimal {
                    println!();
                    println!("You just updated your dotfiles in {}!", "◧ Fig".bold());
                    println!("Automatically applying changes in this terminal.");
                    println!();
                }

                0
            } else {
                if verbosity == UpdatedVerbosity::Full {
                    println!();
                    println!("You just updated your dotfiles in {}!", "◧ Fig".bold());
                    println!(
                        "To apply changes run {} or open a new terminal",
                        "fig source".magenta().bold()
                    );
                    println!();
                }

                1
            }
        }
        Err(_) => 1,
    };

    if let Err(err) = tokio::fs::write(&file, "").await {
        error!("Unable to write to file: {}", err);
    }

    exit(exit_code);
}
