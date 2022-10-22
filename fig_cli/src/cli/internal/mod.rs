pub mod local_state;
use std::fmt::Display;
use std::fs::OpenOptions;
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
    Context,
    ContextCompat,
    Result,
};
use fig_install::InstallComponents;
#[cfg(target_os = "macos")]
use fig_integrations::input_method::InputMethod;
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
use fig_request::auth::get_token;
use fig_request::Request;
use fig_sync::dotfiles::notify::TerminalNotification;
use fig_telemetry::{
    TrackEvent,
    TrackEventType,
    TrackSource,
};
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

use crate::cli::installation::install_cli;

#[derive(Debug, Args, PartialEq, Eq)]
#[command(group(
        ArgGroup::new("output")
            .args(&["filename", "exit_code"])
            .multiple(true)
            .requires_all(&["filename", "exit_code"])
            ))]
pub struct CallbackArgs {
    handler_id: String,
    #[arg(group = "output")]
    filename: Option<String>,
    #[arg(group = "output")]
    exit_code: Option<i64>,
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct InstallArgs {
    /// Install only the daemon
    #[arg(long)]
    pub daemon: bool,
    /// Install only the shell integrations
    #[arg(long)]
    pub dotfiles: bool,
    /// Prompt input method installation
    #[arg(long)]
    pub input_method: bool,
    /// Don't confirm automatic installation.
    #[arg(long)]
    pub no_confirm: bool,
    /// Force installation of fig
    #[arg(long)]
    pub force: bool,
    /// Install only the ssh integration.
    #[arg(long)]
    pub ssh: bool,
}

impl From<InstallArgs> for InstallComponents {
    fn from(args: InstallArgs) -> Self {
        let InstallArgs {
            daemon,
            dotfiles,
            input_method,
            ssh,
            ..
        } = args;
        if daemon || dotfiles || ssh || input_method {
            let mut install_components = InstallComponents::empty();
            install_components.set(InstallComponents::DAEMON, daemon);
            install_components.set(InstallComponents::SHELL_INTEGRATIONS, dotfiles);
            install_components.set(InstallComponents::INPUT_METHOD, input_method);
            install_components.set(InstallComponents::SSH, ssh);
            install_components
        } else {
            InstallComponents::all()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "UPPER")]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum StateComponent {
    Figterm,
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
#[command(hide = true, alias = "_")]
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
        #[arg(long)]
        daemon: bool,
        /// Uninstall only the shell integrations
        #[arg(long)]
        dotfiles: bool,
        /// Uninstall only the input method
        #[arg(long)]
        input_method: bool,
        /// Uninstall only the binary
        #[arg(long)]
        binary: bool,
        /// Uninstall only the ssh integration
        #[arg(long)]
        ssh: bool,
    },
    GetShell,
    Hostname,
    ShouldFigtermLaunch,
    Event {
        /// Name of the event.
        #[arg(long)]
        name: String,
        /// Payload of the event as a JSON string.
        #[arg(long)]
        payload: Option<String>,
        /// Apps to send the event to.
        #[arg(long)]
        apps: Vec<String>,
    },
    AuthToken,
    Request {
        #[arg(long)]
        route: String,
        #[arg(long, default_value_t = Method::Get)]
        method: Method,
        #[arg(long)]
        body: Option<String>,
        #[arg(long)]
        namespace: Option<String>,
        #[arg(long)]
        release: bool,
    },
    SocketsDir,
    StreamFromSocket,
    FigtermSocketPath {
        session_id: String,
    },
    #[command(group(
        ArgGroup::new("target")
            .multiple(false)
            .required(true)
    ))]
    Ipc {
        #[arg(long, group = "target")]
        app: bool,
        #[arg(long, group = "target")]
        daemon: bool,
        #[arg(long, group = "target")]
        figterm: Option<String>,
        #[arg(long)]
        json: String,
        #[arg(long)]
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
    OpenUninstallPage {
        #[arg(long)]
        verbose: bool,
    },
    /// Displays prompt to install remote shell integrations
    PromptSsh {
        remote_dest: String,
    },
    /// Queries remote repository for updates given the specified versions
    QueryIndex {
        #[arg(short, long)]
        channel: String,
        #[arg(short, long)]
        kind: String,
        #[arg(short, long)]
        variant: String,
        #[arg(short = 'e', long)]
        version: String,
        #[arg(short, long)]
        architecture: String,
        #[arg(short = 'r', long)]
        enable_rollout: bool,
    },
    #[cfg(target_os = "macos")]
    AttemptToFinishInputMethodInstallation {
        bundle_path: Option<PathBuf>,
    },
    DumpState {
        component: StateComponent,
    },
}

const BUFFER_SIZE: usize = 1024;

impl InternalSubcommand {
    pub async fn execute(self) -> Result<()> {
        match self {
            InternalSubcommand::Install(args) => {
                let no_confirm = args.no_confirm;
                let force = args.force;
                install_cli(args.into(), no_confirm, force).await?
            },
            InternalSubcommand::Uninstall {
                daemon,
                dotfiles,
                input_method,
                binary,
                ssh,
            } => {
                let components = if daemon || dotfiles || binary || ssh || input_method {
                    let mut uninstall_components = InstallComponents::empty();
                    uninstall_components.set(InstallComponents::DAEMON, daemon);
                    uninstall_components.set(InstallComponents::SHELL_INTEGRATIONS, dotfiles);
                    uninstall_components.set(InstallComponents::INPUT_METHOD, input_method);
                    uninstall_components.set(InstallComponents::BINARY, binary);
                    uninstall_components.set(InstallComponents::SSH, ssh);
                    uninstall_components
                } else {
                    InstallComponents::all()
                };
                if components.contains(InstallComponents::BINARY) {
                    if option_env!("FIG_IS_PACKAGE_MANAGED").is_some() {
                        println!("Uninstall Fig via your package manager");
                    } else {
                        fig_install::uninstall(InstallComponents::BINARY).await?;
                        println!("\n{}\n", "Fig binary has been uninstalled".bold())
                    }
                }

                let mut components = components;
                components.set(InstallComponents::BINARY, false);
                fig_install::uninstall(components).await?;
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
                            use fig_util::process_info::LinuxExt;

                            match (|| {
                                let current_pid = fig_util::process_info::Pid::current();

                                let parent_pid = current_pid.parent()?;
                                let parent_path = parent_pid.exe()?;
                                let parent_name = parent_path.file_name()?.to_str()?;

                                let valid_parent = ["zsh", "bash", "fish", "nu"].contains(&parent_name);

                                if fig_util::system_info::in_ssh() {
                                    if std::env::var_os("FIG_TERM").is_some() {
                                        return Some((false, "❌ In SSH and FIG_TERM is set".into()));
                                    } else {
                                        return Some((true, "✅ In SSH and FIG_TERM is not set".into()));
                                    }
                                }

                                let grandparent_pid = parent_pid.parent()?;
                                let grandparent_path = grandparent_pid.exe()?;
                                let grandparent_name = grandparent_path.file_name()?.to_str()?;
                                let grandparent_cmdline = grandparent_pid.cmdline()?;
                                let grandparent_exe = grandparent_cmdline.split('/').last()?;

                                let valid_grandparent = fig_util::terminal::LINUX_TERMINALS
                                    .iter().chain(fig_util::terminal::SPECIAL_TERMINALS.iter())
                                    .any(|terminal| terminal.executable_names().contains(&grandparent_name)
                                        || terminal.executable_names().contains(&grandparent_exe));

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
                release,
            } => {
                let method = fig_request::Method::from_str(&method.to_string())?;
                let mut request = if release {
                    Request::new_release(method, route)
                } else {
                    Request::new(method, route)
                }
                .namespace(namespace)
                .auth();
                if let Some(body) = body {
                    let value: serde_json::Value = serde_json::from_str(&body)?;
                    request = request.body(value);
                }
                if release {
                    let _ = writeln!(stdout(), "{}", request.raw_text().await?);
                } else {
                    let value = request.json().await?;
                    writeln!(stdout(), "{value}").ok();
                }
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
            InternalSubcommand::SocketsDir => {
                writeln!(stdout(), "{}", directories::sockets_dir_utf8()?).ok();
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
                println!("Uninstalling additional components...");

                let out = Command::new("users").output()?;
                let users = String::from_utf8_lossy(&out.stdout);

                let mut uninstall_success = false;
                let mut open_page_success = false;

                let emit = tokio::spawn(fig_telemetry::emit_track(TrackEvent::new(
                    TrackEventType::UninstalledApp,
                    TrackSource::Cli,
                    env!("CARGO_PKG_VERSION").into(),
                    std::iter::empty::<(&str, &str)>(),
                )));

                for user in users
                    .split('\n')
                    .map(|line| line.trim())
                    .filter(|line| !line.is_empty())
                {
                    if let Ok(exit_status) = tokio::process::Command::new("runuser")
                        .args(["-u", user, "--", "fig", "_", "open-uninstall-page"])
                        .status()
                        .await
                    {
                        if exit_status.success() {
                            open_page_success = true;
                        }
                    }
                    if let Ok(exit_status) = tokio::process::Command::new("runuser")
                        .args(["-u", user, "--", "fig", "integrations", "uninstall", "--silent", "all"])
                        .status()
                        .await
                    {
                        if exit_status.success() {
                            uninstall_success = true;
                        }
                    }
                }

                emit.await.ok();

                if !uninstall_success {
                    bail!("Failed to uninstall properly");
                }

                if !open_page_success {
                    bail!("Failed to uninstall completely");
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
            InternalSubcommand::OpenUninstallPage { verbose } => {
                let url = fig_install::get_uninstall_url();
                if let Err(err) = fig_util::open_url(&url) {
                    if verbose {
                        eprintln!("Failed to open uninstall directly, trying daemon proxy: {err}");
                    }
                    if let Err(err) =
                        fig_ipc::daemon::send_recv_message(fig_proto::daemon::new_open_browser_command(url.clone()))
                            .await
                    {
                        if verbose {
                            eprintln!("Failed to open uninstall via daemon, trying desktop: {err}");
                        }
                        if let Err(err) =
                            fig_ipc::local::send_command_to_socket(fig_proto::local::command::Command::OpenBrowser(
                                fig_proto::local::OpenBrowserCommand { url },
                            ))
                            .await
                        {
                            if verbose {
                                eprintln!("Failed to open uninstall via desktop, no more options: {err}");
                            }
                            std::process::exit(1);
                        }
                    }
                }
            },
            InternalSubcommand::PromptSsh { remote_dest } => {
                if !remote_dest.starts_with("git@") && !remote_dest.starts_with("aur@") {
                    let installed_hosts_file = directories::fig_dir()
                        .context("Can't get fig dir")?
                        .join("ssh_hostnames");
                    let mut installed_hosts = OpenOptions::new()
                        .create(true)
                        .read(true)
                        .append(true)
                        .open(installed_hosts_file)?;

                    let mut contents = String::new();
                    installed_hosts.read_to_string(&mut contents)?;

                    if !contents.contains(&remote_dest) {
                        let bar = format!("╞{}╡", (0..74).map(|_| '═').collect::<String>());
                        println!(
                            "{bar}\n  To install SSH support for {}, run the following on your remote machine\n  \
                            $ curl -fSsL https://fig.io/install-headless.sh | bash\n{bar}",
                            "Fig".magenta(),
                        );
                        let new_line = format!("\n{}", remote_dest);
                        installed_hosts.write_all(&new_line.into_bytes())?;
                    }
                }
            },
            InternalSubcommand::QueryIndex {
                channel,
                kind,
                variant,
                version: current_version,
                architecture,
                enable_rollout,
            } => {
                use fig_install::index::PackageArchitecture;
                use fig_util::manifest::{
                    Channel,
                    Kind,
                    Variant,
                };

                let result = fig_install::index::query_index(
                    Channel::from_str(&channel)?,
                    Kind::from_str(&kind)?,
                    Variant::from_str(&variant)?,
                    &current_version,
                    PackageArchitecture::from_str(&architecture)?,
                    !enable_rollout,
                )
                .await?;

                println!("{result:#?}");
            },
            #[cfg(target_os = "macos")]
            InternalSubcommand::AttemptToFinishInputMethodInstallation { bundle_path } => {
                match InputMethod::finish_input_method_installation(bundle_path) {
                    Ok(_) => exit(0),
                    Err(err) => {
                        println!(
                            "{}",
                            serde_json::to_string(&err).expect("InputMethodError should be serializable")
                        );
                        exit(1)
                    },
                }
            },
            InternalSubcommand::DumpState { .. } => {
                let state =
                    fig_ipc::local::dump_state_command(fig_proto::local::dump_state_command::Type::DumpStateFigterm)
                        .await
                        .context("Failed to send dump state command")?;

                println!("{}", state.json);
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

    let session_id = match std::env::var_os("FIGTERM_SESSION_ID") {
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
    let session_id = match std::env::var("FIGTERM_SESSION_ID") {
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
