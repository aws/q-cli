pub mod local_state;

use std::fs;
use std::io::{
    Read,
    Write,
};
use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;

use anyhow::{
    Context,
    Result,
};
use clap::{
    ArgGroup,
    Args,
    Subcommand,
};
use crossterm::style::Stylize;
use fig_directories::fig_dir;
use fig_ipc::hook::send_hook_to_socket;
use fig_proto::hooks::new_callback_hook;
use native_dialog::{
    MessageDialog,
    MessageType,
};
use rand::distributions::{
    Alphanumeric,
    DistString,
};
use rand::seq::IteratorRandom;
use sysinfo::SystemExt;
use tracing::{
    debug,
    error,
    info,
    trace,
};
use viu::{
    run,
    Config,
};

use crate::cli::installation::{
    self,
    InstallComponents,
};
use crate::dotfiles::notify::TerminalNotification;
use crate::util::get_parent_process_exe;

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
    #[clap(long, conflicts_with_all = &["input-method"])]
    pub daemon: bool,
    /// Install only the shell integrations
    #[clap(long, conflicts_with_all = &["input-method"])]
    pub dotfiles: bool,
    /// Prompt input method installation
    #[clap(long, conflicts_with_all = &["daemon", "dotfiles"])]
    pub input_method: bool,
    /// Don't confirm automatic installation.
    #[clap(long)]
    pub no_confirm: bool,
    /// Force installation of fig
    #[clap(long)]
    pub force: bool,
    /// Install only the ssh integration.
    #[clap(long)]
    pub ssh: bool,
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
    /// Prompt the user that the dotfiles have changes
    /// Also use for `fig source` internals
    PromptDotfilesChanged,
    /// Change the local-state file
    LocalState(local_state::LocalStateArgs),
    /// Callback used for the internal psudoterminal
    Callback(CallbackArgs),
    /// Install fig cli
    Install(InstallArgs),
    InstallIbus {
        fig_ibus_engine_location: String,
    },
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
        /// Uninstall only the ssh integration
        #[clap(long)]
        ssh: bool,
    },
    /// Notify the user that they are uninstalling incorrectly
    WarnUserWhenUninstallingIncorrectly,
    Animation(AnimationArgs),
    GetShell,
    Hostname,
}

pub fn install_cli_from_args(install_args: InstallArgs) -> Result<()> {
    let InstallArgs {
        daemon,
        dotfiles,
        no_confirm,
        force,
        ssh,
        ..
    } = install_args;
    let install_components = if daemon || dotfiles || ssh {
        let mut install_components = InstallComponents::empty();
        install_components.set(InstallComponents::DAEMON, daemon);
        install_components.set(InstallComponents::DOTFILES, dotfiles);
        install_components.set(InstallComponents::SSH, ssh);
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
            InternalSubcommand::InstallIbus {
                fig_ibus_engine_location,
            } => {
                let xml = format!(
                    "<?xml version=\"1.0\" encoding=\"utf-8\" ?>
<component>
    <name>org.freedesktop.IBus.FigIBusEngine</name>\
                     
    <description>Fig integration for the IBus input method</description>
    <version>0.1.0</version>\
                     
    <license></license>
    <author>Fig</author>
    <homepage>https://fig.io</homepage>
    <exec>{fig_ibus_engine_location}</exec>
    <textdomain></textdomain>
    <engines>
        <engine>
            <name>FigIBusEngine</name>
            <longname>Fig IBus Engine</longname>
            <description>Fig integration for the IBus input method</description>
            <author>Fig</author>
        </engine>
    </engines>
</component>"
                );
                tokio::fs::create_dir_all("/usr/share/ibus/component").await?;
                tokio::fs::write("/usr/share/ibus/component/engine.xml", xml).await?;
            },
            InternalSubcommand::Uninstall {
                daemon,
                dotfiles,
                binary,
                ssh,
            } => {
                let uninstall_components = if daemon || dotfiles || binary || ssh {
                    let mut uninstall_components = InstallComponents::empty();
                    uninstall_components.set(InstallComponents::DAEMON, daemon);
                    uninstall_components.set(InstallComponents::DOTFILES, dotfiles);
                    uninstall_components.set(InstallComponents::BINARY, binary);
                    uninstall_components.set(InstallComponents::SSH, ssh);
                    uninstall_components
                } else {
                    InstallComponents::all()
                };

                installation::uninstall_cli(uninstall_components)?
            },
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
                    },
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
                            trace!("Read {} bytes\n{}", size, std::str::from_utf8(&buffer[..size])?);
                        }

                        let filename: String = tmp_path.to_str().context("invalid file path")?.into();
                        trace!("Done reading from stdin!");
                        (filename, -1)
                    },
                };
                let hook = new_callback_hook(&handler_id, &filename, exit_code);

                info!(
                    "Sending 'handlerId: {}, filename: {}, exitcode: {}' over unix socket!\n",
                    handler_id, filename, exit_code
                );

                match send_hook_to_socket(hook).await {
                    Ok(()) => {
                        debug!("Successfully sent hook");
                    },
                    Err(e) => {
                        debug!("Couldn't send hook {}", e);
                    },
                }
            },
            InternalSubcommand::WarnUserWhenUninstallingIncorrectly => {
                MessageDialog::new()
                    .set_type(MessageType::Warning)
                    .set_title("Trying to uninstall Fig?")
                    .set_text("Please run `fig uninstall` rather than moving the app to the Trash.")
                    .show_alert()
                    .unwrap();
            },
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
                                },
                                Err(e) => {
                                    eprintln!("{}", e);
                                    std::process::exit(1);
                                },
                            }
                        }

                        animations_folder.join(fname).into_os_string().into_string().unwrap()
                    },
                    None => {
                        eprintln!("filename cannot be empty");
                        std::process::exit(1);
                    },
                };

                let loading_message = match before_text {
                    Some(t) => t.magenta(),
                    None => String::new().reset(),
                };

                let cleanup_message = match after_text {
                    Some(t) => t.magenta(),
                    None => String::new().reset(),
                };

                // viu stuff to initialize
                let files = vec![path.as_str()];

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
                    &loading_message,
                    &cleanup_message,
                );

                // run animation
                if let Err(e) = run(conf).await {
                    eprintln!("{:?}", e);
                    std::process::exit(1);
                }
            },
            InternalSubcommand::GetShell => {
                if let Ok(exe) = get_parent_process_exe() {
                    print!("{}", exe.display())
                } else {
                    exit(1);
                }
            },
            InternalSubcommand::Hostname => {
                if let Some(hostname) = sysinfo::System::new().host_name() {
                    println!("{}", hostname);
                } else {
                    exit(1);
                }
            },
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

    let session_id = match std::env::var_os("TERM_SESSION_ID") {
        Some(session_id) => session_id,
        None => exit(1),
    };

    let file = std::env::temp_dir()
        .join("fig")
        .join("dotfiles_updates")
        .join(session_id);

    let file_clone = file.clone();
    ctrlc::set_handler(move || {
        crossterm::execute!(std::io::stdout(), crossterm::cursor::Show).ok();
        std::fs::write(&file_clone, "").ok();

        exit(1);
    })
    .ok();

    let file_content = match tokio::fs::read_to_string(&file).await {
        Ok(content) => content,
        Err(_) => {
            if let Err(err) = tokio::fs::create_dir_all(&file.parent().expect("Unable to create parent dir")).await {
                error!("Unable to create directory: {}", err);
            }

            if let Err(err) = tokio::fs::write(&file, "").await {
                error!("Unable to write to file: {}", err);
            }

            exit(1);
        },
    };

    let exit_code = match TerminalNotification::from_str(&file_content) {
        Ok(TerminalNotification::Source) => {
            println!();
            println!("{}", "✅ Dotfiles sourced!".bold());
            println!();

            0
        },
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

            let source_immediately = fig_settings::settings::get_value("dotfiles.sourceImmediately")
                .ok()
                .flatten()
                .and_then(|s| s.as_str().map(|s| s.to_owned()));

            let source_updates = match source_immediately.as_deref() {
                Some("always") => true,
                // Ask is depercated
                // Some("ask") => {
                //     let dialog_result =  dialoguer::Select::with_theme(&dialoguer_theme())
                //             .with_prompt("In the future, would you like Fig to auto-apply dotfiles changes in open
                // terminals?")             .items(&["Yes", "No"])
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
        },
        Err(_) => 1,
    };

    if let Err(err) = tokio::fs::write(&file, "").await {
        error!("Unable to write to file: {}", err);
    }

    exit(exit_code);
}
