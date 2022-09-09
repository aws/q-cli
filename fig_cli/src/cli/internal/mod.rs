pub mod local_state;
use std::fmt::Display;
use std::io::{
    stderr,
    stdout,
    Read,
    Write,
};
use std::path::PathBuf;
use std::process::{
    exit,
    Command,
};
use std::str::FromStr;
use std::time::Duration;

use bytes::{
    Buf,
    BytesMut,
};
use cfg_if::cfg_if;
use clap::{
    ArgGroup,
    Args,
    Subcommand,
    ValueEnum,
};
use crossterm::style::Stylize;
use eyre::{
    bail,
    ContextCompat,
    Result,
};
use fig_auth::get_token;
use fig_install::dotfiles::notify::TerminalNotification;
use fig_ipc::local::send_hook_to_socket;
use fig_ipc::{
    BufferedUnixStream,
    SendMessage,
    SendRecvMessage,
};
use fig_proto::figterm::figterm_request_message::Request as FigtermRequest;
use fig_proto::figterm::{
    FigtermRequestMessage,
    UpdateShellContextRequest,
};
use fig_proto::hooks::{
    new_callback_hook,
    new_event_hook,
};
use fig_proto::local::EnvironmentVariable;
use fig_proto::ReflectMessage;
use fig_request::Request;
use fig_util::directories::figterm_socket_path;
use fig_util::{
    directories,
    get_parent_process_exe,
};
use rand::distributions::{
    Alphanumeric,
    DistString,
};
use sysinfo::{
    System,
    SystemExt,
};
use tokio::io::{
    AsyncReadExt,
    AsyncWriteExt,
};
use tokio::select;
use tracing::{
    debug,
    error,
    info,
    trace,
};

use crate::cli::installation::{
    self,
    InstallComponents,
};

#[derive(Debug, Args)]
#[clap(group(
        ArgGroup::new("output")
            .args(&["filename", "exit-code"])
            .multiple(true)
            .requires_all(&["filename", "exit-code"])
            ))]
pub struct CallbackArgs {
    #[clap(value_parser)]
    handler_id: String,
    #[clap(value_parser, group = "output")]
    filename: Option<String>,
    #[clap(value_parser, group = "output")]
    exit_code: Option<i64>,
}

#[derive(Debug, Args)]
pub struct InstallArgs {
    /// Install only the daemon
    #[clap(long, value_parser, conflicts_with_all = &["input-method"])]
    pub daemon: bool,
    /// Install only the shell integrations
    #[clap(long, value_parser, conflicts_with_all = &["input-method"])]
    pub dotfiles: bool,
    /// Prompt input method installation
    #[clap(long, value_parser, conflicts_with_all = &["daemon", "dotfiles"])]
    pub input_method: bool,
    /// Don't confirm automatic installation.
    #[clap(long, value_parser)]
    pub no_confirm: bool,
    /// Force installation of fig
    #[clap(long, value_parser)]
    pub force: bool,
    /// Install only the ssh integration.
    #[clap(long, value_parser)]
    pub ssh: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[clap(rename_all = "UPPER")]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Connect,
    Patch,
    Trace,
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Head => "HEAD",
            Method::Options => "OPTIONS",
            Method::Connect => "CONNECT",
            Method::Patch => "PATCH",
            Method::Trace => "TRACE",
        })
    }
}

#[derive(Debug, Subcommand)]
#[clap(hide = true, alias = "_")]
pub enum InternalSubcommand {
    /// Prompt the user that the dotfiles have changes
    /// Also use for `fig source` internals
    PromptDotfilesChanged,
    /// Command that is run during the PreCmd section of
    /// the fig integrations.
    PreCmd,
    /// Change the local-state file
    LocalState(local_state::LocalStateArgs),
    /// Callback used for the internal pseudoterminal
    Callback(CallbackArgs),
    /// Install fig cli
    Install(InstallArgs),
    /// Uninstall fig cli
    Uninstall {
        /// Uninstall only the daemon
        #[clap(long, value_parser)]
        daemon: bool,
        /// Uninstall only the shell integrations
        #[clap(long, value_parser)]
        dotfiles: bool,
        /// Uninstall only the binary
        #[clap(long, value_parser)]
        binary: bool,
        /// Uninstall only the ssh integration
        #[clap(long, value_parser)]
        ssh: bool,
    },
    GetShell,
    Hostname,
    ShouldFigtermLaunch,
    Event {
        /// Name of the event.
        #[clap(long, value_parser)]
        name: String,
        /// Payload of the event as a JSON string.
        #[clap(long, value_parser)]
        payload: Option<String>,
        /// Apps to send the event to.
        #[clap(long, value_parser)]
        apps: Vec<String>,
    },
    AuthToken,
    Request {
        #[clap(long, value_parser)]
        route: String,
        #[clap(long, value_parser, default_value_t = Method::Get)]
        method: Method,
        #[clap(long, value_parser)]
        body: Option<String>,
        #[clap(long, value_parser)]
        namespace: Option<String>,
    },
    FigSocketPath,
    StreamFromSocket,
    FigtermSocketPath {
        session_id: String,
    },
    #[clap(group(
        ArgGroup::new("target")
            .multiple(false)
            .required(true)
    ))]
    Ipc {
        #[clap(long, value_parser, group = "target")]
        app: bool,
        #[clap(long, value_parser, group = "target")]
        daemon: bool,
        #[clap(long, value_parser, group = "target")]
        figterm: Option<String>,
        #[clap(long, value_parser)]
        json: String,
        #[clap(long, value_parser)]
        recv: bool,
    },
    /// Linux only
    UninstallForAllUsers,
    Uuidgen,
    #[cfg(target_os = "linux")]
    IbusBootstrap,
    #[cfg(target_os = "linux")]
    /// Checks for sandboxing
    DetectSandbox,
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
            InternalSubcommand::PreCmd => pre_cmd().await,
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
            InternalSubcommand::GetShell => {
                if let Some(exe) = get_parent_process_exe() {
                    #[cfg(windows)]
                    let exe = if exe.file_name().unwrap() == "bash.exe" {
                        exe.parent()
                            .context("No parent")?
                            .parent()
                            .context("No parent")?
                            .parent()
                            .context("No parent")?
                            .join("bin")
                            .join("bash.exe")
                    } else {
                        exe
                    };

                    if write!(stdout(), "{}", exe.display()).is_ok() {
                        return Ok(());
                    }
                }
                exit(1);
            },
            InternalSubcommand::Hostname => {
                if let Some(hostname) = System::new().host_name() {
                    if write!(stdout(), "{hostname}").is_ok() {
                        return Ok(());
                    }
                }
                exit(1);
            },
            InternalSubcommand::ShouldFigtermLaunch => {
                // Exit code:
                //   - 0 execute figterm
                //   - 1 dont execute figterm
                //   - 2 fallback to FIG_TERM env
                cfg_if!(
                    if #[cfg(target_os = "linux")] {
                        if fig_util::system_info::in_wsl() {
                            exit(2)
                        } else {
                            use fig_util::process_info::PidExt;
                            match (|| {
                                let current_pid = fig_util::process_info::Pid::current();

                                let parent_pid = current_pid.parent()?;
                                let parent_path = parent_pid.exe()?;
                                let parent_name = parent_path.file_name()?.to_str()?;

                                let valid_parent = ["zsh", "bash", "fish", "nu"].contains(&parent_name);

                                let grandparent_pid = parent_pid.parent()?;
                                let grandparent_path = grandparent_pid.exe()?;
                                let grandparent_name = grandparent_path.file_name()?.to_str()?;

                                let valid_grandparent = fig_util::terminal::LINUX_TERMINALS
                                    .iter().chain(fig_util::terminal::SPECIAL_TERMINALS.iter())
                                    .any(|terminal| terminal.executable_names().contains(&grandparent_name));

                                let ancestry = format!(
                                    "{} {} ({grandparent_pid}) <- {} {} ({parent_pid})",
                                    if valid_grandparent { "✅" } else { "❌" },
                                    grandparent_path.display(),
                                    if valid_parent { "✅" } else { "❌" },
                                    parent_path.display(),
                                );

                                Some((valid_parent && valid_grandparent, ancestry))
                            })() {
                                Some((should_execute, ancestry)) => {
                                    writeln!(stdout(), "{ancestry}").ok();
                                    exit(if should_execute { 0 } else { 1 });
                                },
                                None => exit(1),
                            }
                        }
                    } else if #[cfg(target_os = "windows")] {
                        use winapi::um::consoleapi::GetConsoleMode;
                        use std::os::windows::io::AsRawHandle;

                        let mut mode = 0;
                        let stdin_ok = unsafe {
                            GetConsoleMode(
                                std::io::stdin().as_raw_handle() as *mut _,
                                &mut mode
                            )
                        };
                        exit(if stdin_ok == 1 { 2 } else { 1 });
                    } else {
                        exit(2);
                    }
                );
            },
            InternalSubcommand::Event { payload, apps, name } => {
                let hook = new_event_hook(name, payload, apps);
                send_hook_to_socket(hook).await?;
            },
            InternalSubcommand::AuthToken => {
                writeln!(stdout(), "{}", get_token().await?).ok();
            },
            InternalSubcommand::Request {
                route,
                method,
                body,
                namespace,
            } => {
                let method = fig_request::Method::from_str(&method.to_string())?;
                let mut request = Request::new(method, route).namespace(namespace);
                if let Some(body) = body {
                    let value: serde_json::Value = serde_json::from_str(&body)?;
                    request = request.body(value);
                }
                let value = request.auth().json().await?;
                writeln!(stdout(), "{value}").ok();
            },
            InternalSubcommand::Ipc {
                app,
                daemon,
                figterm,
                json,
                recv,
            } => {
                let message = fig_proto::FigMessage::json(serde_json::from_str::<serde_json::Value>(&json)?)?;

                let socket = if app {
                    directories::fig_socket_path().expect("Failed to get socket path")
                } else if daemon {
                    directories::daemon_socket_path().expect("Failed to get daemon socket path")
                } else if let Some(ref figterm) = figterm {
                    directories::figterm_socket_path(figterm).expect("Failed to get socket path")
                } else {
                    bail!("No destination for message");
                };

                let mut conn = BufferedUnixStream::connect(socket).await?;

                if recv {
                    macro_rules! recv {
                        ($abc:path) => {{
                            let response: Option<$abc> = conn.send_recv_message(message).await?;
                            match response {
                                Some(response) => {
                                    let message = response.transcode_to_dynamic();
                                    println!("{}", serde_json::to_string(&message)?)
                                },
                                None => bail!("Received EOF while waiting for response"),
                            }
                        }};
                    }

                    if app {
                        recv!(fig_proto::local::CommandResponse);
                    } else if daemon {
                        recv!(fig_proto::daemon::DaemonResponse);
                    } else if figterm.is_some() {
                        recv!(fig_proto::figterm::FigtermResponseMessage);
                    }
                } else {
                    conn.send_message(message).await?;
                }
            },
            InternalSubcommand::FigSocketPath => {
                writeln!(stdout(), "{}", directories::fig_socket_path()?.to_string_lossy()).ok();
            },
            InternalSubcommand::FigtermSocketPath { session_id } => {
                writeln!(
                    stdout(),
                    "{}",
                    directories::figterm_socket_path(session_id)?.to_string_lossy()
                )
                .ok();
            },
            InternalSubcommand::UninstallForAllUsers => {
                let out = Command::new("users").output()?;
                let users = String::from_utf8_lossy(&out.stdout);
                for user in users
                    .split('\n')
                    .map(|line| line.trim())
                    .filter(|line| !line.is_empty())
                {
                    Command::new("sudo")
                        .args(&["-u", user, "--", "fig", "integrations", "uninstall", "--silent", "all"])
                        .spawn()?
                        .wait()?;
                }
            },
            InternalSubcommand::StreamFromSocket => {
                let mut stdout = tokio::io::stdout();
                let mut stdin = tokio::io::stdin();

                let mut stdout_buf = BytesMut::with_capacity(1024);
                let mut stream_buf = BytesMut::with_capacity(1024);

                let socket = directories::secure_socket_path()?;
                while let Ok(mut stream) = BufferedUnixStream::connect_timeout(&socket, Duration::from_secs(5)).await {
                    loop {
                        select! {
                            n = stream.read_buf(&mut stdout_buf) => {
                                match n {
                                    Ok(0) | Err(_) => {
                                        break;
                                    }
                                    Ok(mut n) => {
                                        while !stdout_buf.is_empty() {
                                            let m = stdout.write(&stdout_buf[..n]).await?;
                                            stdout.flush().await?;
                                            stdout_buf.advance(m);
                                            n -= m;
                                        }
                                        stdout_buf.clear();
                                    }
                                }
                            }
                            n = stdin.read_buf(&mut stream_buf) => {
                                match n {
                                    Ok(0) | Err(_) => {
                                        break;
                                    }
                                    Ok(mut n) => {
                                        while !stream_buf.is_empty() {
                                            let m = stream.write(&stream_buf[..n]).await?;
                                            stream.flush().await?;
                                            stream_buf.advance(m);
                                            n -= m;
                                        }
                                        stream_buf.clear();
                                    }
                                }
                            }
                        }
                    }
                }
            },
            InternalSubcommand::Uuidgen => {
                writeln!(stdout(), "{}", uuid::Uuid::new_v4()).ok();
            },
            #[cfg(target_os = "linux")]
            InternalSubcommand::IbusBootstrap => {
                use sysinfo::{
                    ProcessRefreshKind,
                    RefreshKind,
                };
                use tokio::process::Command;

                let system = tokio::task::block_in_place(|| {
                    System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()))
                });
                if system.processes_by_name("ibus-daemon").next().is_none() {
                    info!("Launching 'ibus-daemon'");
                    match Command::new("ibus-daemon").arg("-drxR").output().await {
                        Ok(std::process::Output { status, stdout, stderr }) if !status.success() => {
                            let stdout = String::from_utf8_lossy(&stdout);
                            let stderr = String::from_utf8_lossy(&stderr);
                            eyre::bail!(
                                "Failed to run 'ibus-daemon -drxR': status={status:?} stdout={stdout:?} stderr={stderr:?}"
                            );
                        },
                        Err(err) => eyre::bail!("Failed to run 'ibus-daemon -drxR': {err}"),
                        Ok(_) => writeln!(stdout(), "ibus-daemon is now running").ok(),
                    };
                } else {
                    writeln!(stdout(), "ibus-daemon is already running").ok();
                }
            },
            #[cfg(target_os = "linux")]
            InternalSubcommand::DetectSandbox => {
                use fig_util::system_info::linux::SandboxKind;
                match fig_util::system_info::linux::detect_sandbox() {
                    SandboxKind::None => println!("No sandbox detected"),
                    SandboxKind::Flatpak => println!("You are in a Flatpak"),
                    SandboxKind::Snap => println!("You are in a Snap"),
                    SandboxKind::Docker => println!("You are in a Docker container"),
                    SandboxKind::Container(None) => println!("You are in a generic container"),
                    SandboxKind::Container(Some(engine)) => println!("You are in a {engine} container"),
                };
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
                error!("Unable to create directory: {err}");
            }

            if let Err(err) = tokio::fs::write(&file, "").await {
                error!("Unable to write to file: {err}");
            }

            exit(1);
        },
    };

    let exit_code = match TerminalNotification::from_str(&file_content) {
        Ok(TerminalNotification::Source) => {
            writeln!(stdout(), "\n{}\n", "✅ Dotfiles sourced!".bold()).ok();
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

            let source_immediately = match fig_settings::settings::get_value("dotfiles.sourceImmediately")
                .ok()
                .flatten()
            {
                Some(serde_json::Value::String(s)) => Some(s),
                _ => None,
            };

            let source_updates = matches!(source_immediately.as_deref(), Some("always"));

            if source_updates {
                if verbosity >= UpdatedVerbosity::Minimal {
                    writeln!(
                        stdout(),
                        "\nYou just updated your dotfiles in {}!\nAutomatically applying changes in this terminal.\n",
                        "◧ Fig".bold()
                    )
                    .ok();
                }
                0
            } else {
                if verbosity == UpdatedVerbosity::Full {
                    writeln!(
                        stdout(),
                        "\nYou just updated your dotfiles in {}!\nTo apply changes run {} or open a new terminal",
                        "◧ Fig".bold(),
                        "fig source".magenta().bold()
                    )
                    .ok();
                }
                1
            }
        },
        Err(_) => 1,
    };

    if let Err(err) = tokio::fs::write(&file, "").await {
        error!("Unable to write to file: {err}");
    }

    exit(exit_code);
}

pub async fn pre_cmd() {
    let session_id = match std::env::var("TERM_SESSION_ID") {
        Ok(session_id) => session_id,
        Err(_) => exit(1),
    };

    let session_id_clone = session_id.clone();
    let shell_state_join = tokio::spawn(async move {
        let session_id = session_id_clone;
        match figterm_socket_path(&session_id) {
            Ok(figterm_path) => match fig_ipc::socket_connect(figterm_path).await {
                Ok(mut figterm_stream) => {
                    let message = FigtermRequestMessage {
                        request: Some(FigtermRequest::UpdateShellContext(UpdateShellContextRequest {
                            update_environment_variables: true,
                            environment_variables: std::env::vars()
                                .map(|(key, value)| EnvironmentVariable {
                                    key,
                                    value: Some(value),
                                })
                                .collect(),
                        })),
                    };
                    if let Err(err) = figterm_stream.send_message(message).await {
                        error!(%err, %session_id, "Failed to send UpdateShellContext to Figterm");
                    }
                },
                Err(err) => error!(%err, %session_id, "Failed to connect to Figterm socket"),
            },
            Err(err) => error!(%err, %session_id, "Failed to get Figterm socket path"),
        }
    });

    let notification_join = tokio::spawn(async move {
        let file = std::env::temp_dir()
            .join("fig")
            .join("dotfiles_updates")
            .join(session_id);

        let file_content = match tokio::fs::read_to_string(&file).await {
            Ok(content) => content,
            Err(_) => {
                if let Err(err) = tokio::fs::create_dir_all(&file.parent().expect("Unable to create parent dir")).await
                {
                    error!("Unable to create directory: {err}");
                }

                if let Err(err) = tokio::fs::write(&file, "").await {
                    error!("Unable to write to file: {err}");
                }

                exit(1);
            },
        };

        match TerminalNotification::from_str(&file_content) {
            Ok(TerminalNotification::Source) => {
                writeln!(stdout(), "EXEC_NEW_SHELL").ok();
                writeln!(stderr(), "\n{}\n", "✅ Dotfiles sourced!".bold()).ok();
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

                let source_immediately = match fig_settings::settings::get_value("dotfiles.sourceImmediately")
                    .ok()
                    .flatten()
                {
                    Some(serde_json::Value::String(s)) => Some(s),
                    _ => None,
                };

                let source_updates = matches!(source_immediately.as_deref(), Some("always"));

                if source_updates {
                    if verbosity >= UpdatedVerbosity::Minimal {
                        writeln!(
                        stderr(),
                        "\nYou just updated your dotfiles in {}!\nAutomatically applying changes in this terminal.\n",
                        "◧ Fig".bold()
                    )
                    .ok();
                    }
                    0
                } else {
                    if verbosity == UpdatedVerbosity::Full {
                        writeln!(
                            stderr(),
                            "\nYou just updated your dotfiles in {}!\nTo apply changes run {} or open a new terminal",
                            "◧ Fig".bold(),
                            "fig source".magenta().bold()
                        )
                        .ok();
                    }
                    1
                }
            },
            Err(_) => 1,
        };

        if let Err(err) = tokio::fs::write(&file, "").await {
            error!("Unable to write to file: {err}");
        }
    });

    let (shell_state, notification) = tokio::join!(shell_state_join, notification_join);

    shell_state.ok();
    notification.ok();

    exit(0);
}
