pub mod local_state;

use super::source::TerminalNotification;
use crate::cli::installation::{self, InstallComponents};

use anyhow::{Context, Result};
use clap::{ArgGroup, Args, Subcommand};
use crossterm::style::Stylize;
use fig_directories::fig_dir;
use fig_ipc::hook::send_hook_to_socket;
use fig_proto::hooks::new_callback_hook;
use native_dialog::{MessageDialog, MessageType};
use rand::distributions::{Alphanumeric, DistString};
use rand::seq::IteratorRandom;
use std::{
    fs,
    io::{Read, Write},
    path::PathBuf,
    process::exit,
    str::FromStr,
};
use tracing::{debug, error, info, trace};
use viu::{Config, run};

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

#[derive(Debug, Args)]
pub struct AnimationArgs {
    // resource to play
    #[clap(short, long)]
    filename: Option<String>,

    // framerate to play the GIF with
    #[clap(short, long)]
    rate: Option<i32>,

    // text to print before GIF/img appears
    #[clap(short, long)]
    before_text: Option<String>,

    // text to print before GIF/img disappears
    #[clap(short, long)]
    after_text: Option<String>,
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

    Animation(AnimationArgs),
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
            InternalSubcommand::Animation(AnimationArgs {
                filename,
                rate,
                before_text,
                after_text,
            }) => {
                let path = match filename {
                    Some(mut fname) => {
                        let animations_folder = fig_dir().unwrap().join("animations");
                        if fname == "random" {
                            // pick a random animation file from animations folder
                            let paths = fs::read_dir(&animations_folder).unwrap();
                            match paths.choose(&mut rand::thread_rng()).unwrap() {
                                Ok(p) => {
                                    fname = p.file_name().into_string().unwrap();
                                }
                                Err(e) => {
                                    eprintln!("{}", e);
                                    std::process::exit(1);
                                }
                            }
                        }

                        animations_folder
                            .join(fname)
                            .into_os_string()
                            .into_string()
                            .unwrap()
                    }
                    None => {
                        eprintln!("filename cannot be empty");
                        std::process::exit(1);
                    }
                };


                let green = "\x1b[0;32m";
                let purple = "\x1b[38;5;171m";
                let loading_message = match before_text {
                    Some(t) => {
                        let s = format!("{}{}", green, t);
                        s
                    }
                    None => format!("{}😀 Loading GIF...", green)
                };

                let cleanup_message = match after_text {
                    Some(t) => {
                        let s = format!("{}{}", purple, t);
                        s
                    }
                    None => String::new(),
                };

                // viu stuff to initialize
                let mut files = Vec::new();
                files.push(path.as_str());
                let conf = Config::new(
                    None,
                    None,
                    Some(files),
                    false,
                    false,
                    false,
                    true,
                    false,
                    false,
                    rate,
                    loading_message.as_str(),
                    cleanup_message.as_str(),
                );

                // run animation
                if let Err(e) = run(conf) {
                    eprintln!("{:?}", e);
                    std::process::exit(1);
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
                _ => UpdatedVerbosity::Full,
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
