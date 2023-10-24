pub mod local_state;
pub mod should_figterm_launch;
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
use std::time::Duration;

use bytes::{
    Buf,
    BytesMut,
};
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
    FigtermResponseMessage,
    NotifySshSessionStartedRequest,
    UpdateShellContextRequest,
};
use fig_proto::hooks::{
    new_callback_hook,
    new_event_hook,
};
use fig_proto::local::EnvironmentVariable;
use fig_proto::ReflectMessage;
use fig_util::desktop::{
    launch_fig_desktop,
    LaunchArgs,
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
    WebNotifications,
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
#[command(hide = true, alias = "_")]
pub enum InternalSubcommand {
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
    /// Detects if Figterm should be launched
    ///
    /// Exit code:
    /// - 0 execute figterm
    /// - 1 dont execute figterm
    /// - 2 fallback to CW_TERM env
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
    SshLocalCommand {
        remote_dest: String,
        uuid: String,
    },
    /// \[Deprecated\] Displays prompt to install remote shell integrations.
    PromptSsh {
        remote_dest: String,
    },
    #[cfg(target_os = "macos")]
    AttemptToFinishInputMethodInstallation {
        bundle_path: Option<PathBuf>,
    },
    DumpState {
        component: StateComponent,
    },
    FinishUpdate {
        #[arg(long)]
        relaunch_dashboard: bool,
    },
    #[cfg(target_os = "macos")]
    SwapFiles {
        from: PathBuf,
        to: PathBuf,
    },
    #[deprecated]
    #[command(hide = true)]
    CheckSSH {
        remote_username: String,
    },
    #[cfg(target_os = "macos")]
    BrewUninstall {
        #[arg(long)]
        zap: bool,
    },
    /// Generates an SSH configuration file
    ///
    /// This lets us bypass a bug in Include and vdollar_expand that causes environment variables to
    /// be expanded, even in files that are only referenced in match blocks that resolve to false
    GenerateSSH {
        remote_username: String,
    },
    GhostText {
        #[arg(long, allow_hyphen_values = true)]
        buffer: String,
    },
    GhostTextAccept {
        #[arg(long, allow_hyphen_values = true)]
        buffer: String,
        #[arg(long, allow_hyphen_values = true)]
        suggestion: String,
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
                        let tmp_filename = format!("fig-callback-{file_id}");
                        let tmp_path = PathBuf::from("/tmp").join(tmp_filename);
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
            InternalSubcommand::GetShell => {},
            InternalSubcommand::Hostname => {
                if let Some(hostname) = System::new().host_name() {
                    if write!(stdout(), "{hostname}").is_ok() {
                        return Ok(());
                    }
                }
                exit(1);
            },
            InternalSubcommand::ShouldFigtermLaunch => should_figterm_launch::should_figterm_launch(),
            InternalSubcommand::Event { payload, apps, name } => {
                let hook = new_event_hook(name, payload, apps);
                send_hook_to_socket(hook).await?;
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

                // let emit = tokio::spawn(fig_telemetry::emit_track(TrackEvent::new(
                //     TrackEventType::UninstalledApp,
                //     TrackSource::Cli,
                //     env!("CARGO_PKG_VERSION").into(),
                //     std::iter::empty::<(&str, &str)>(),
                // )));

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

                // emit.await.ok();

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

                let socket = directories::remote_socket_path()?;
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
                exit(match fig_util::system_info::linux::detect_sandbox() {
                    SandboxKind::None => {
                        println!("No sandbox detected");
                        0
                    },
                    SandboxKind::Flatpak => {
                        println!("You are in a Flatpak");
                        1
                    },
                    SandboxKind::Snap => {
                        println!("You are in a Snap");
                        1
                    },
                    SandboxKind::Docker => {
                        println!("You are in a Docker container");
                        1
                    },
                    SandboxKind::Container(None) => {
                        println!("You are in a generic container");
                        1
                    },
                    SandboxKind::Container(Some(engine)) => {
                        println!("You are in a {engine} container");
                        1
                    },
                })
            },
            InternalSubcommand::OpenUninstallPage { verbose } => {
                let url = fig_install::get_uninstall_url(false);
                if let Err(err) = fig_util::open_url(&url) {
                    if verbose {
                        eprintln!("Failed to open uninstall directly, trying daemon proxy: {err}");
                    }

                    if let Err(err) = fig_ipc::local::send_command_to_socket(
                        fig_proto::local::command::Command::OpenBrowser(fig_proto::local::OpenBrowserCommand { url }),
                    )
                    .await
                    {
                        if verbose {
                            eprintln!("Failed to open uninstall via desktop, no more options: {err}");
                        }
                        std::process::exit(1);
                    }
                }
            },
            InternalSubcommand::PromptSsh { .. } => {},
            InternalSubcommand::SshLocalCommand { remote_dest, uuid } => {
                // Ensure desktop app is running to avoid SSH errors on stdout when local side of
                // RemoteForward isn't listening
                launch_fig_desktop(LaunchArgs {
                    wait_for_socket: true,
                    open_dashboard: false,
                    immediate_update: false,
                    verbose: false,
                })
                .ok();

                if let Ok(session_id) = std::env::var("CWTERM_SESSION_ID") {
                    let mut conn =
                        BufferedUnixStream::connect(fig_util::directories::figterm_socket_path(&session_id)?).await?;
                    conn.send_message(FigtermRequestMessage {
                        request: Some(FigtermRequest::NotifySshSessionStarted(
                            NotifySshSessionStartedRequest {
                                uuid,
                                remote_host: remote_dest,
                            },
                        )),
                    })
                    .await?;
                }
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
            InternalSubcommand::DumpState { component } => {
                use fig_proto::local::dump_state_command::Type as StateCommandType;

                let state = fig_ipc::local::dump_state_command(match component {
                    StateComponent::Figterm => StateCommandType::DumpStateFigterm,
                    StateComponent::WebNotifications => StateCommandType::DumpStateWebNotifications,
                })
                .await
                .context("Failed to send dump state command")?;

                println!("{}", state.json);
            },
            InternalSubcommand::FinishUpdate { relaunch_dashboard } => {
                // Wait some time for the previous installation to close
                tokio::time::sleep(Duration::from_millis(100)).await;

                crate::util::quit_fig(false).await.ok();

                tokio::time::sleep(Duration::from_millis(200)).await;

                launch_fig_desktop(LaunchArgs {
                    wait_for_socket: false,
                    open_dashboard: relaunch_dashboard,
                    immediate_update: false,
                    verbose: false,
                })
                .ok();
            },
            #[cfg(target_os = "macos")]
            InternalSubcommand::SwapFiles { from, to } => {
                use std::os::unix::prelude::OsStrExt;

                let from_cstr = match std::ffi::CString::new(from.as_os_str().as_bytes()).context("Invalid from path") {
                    Ok(cstr) => cstr,
                    Err(err) => {
                        writeln!(stderr(), "Invalid from path: {err}").ok();
                        std::process::exit(1);
                    },
                };

                let to_cstr = match std::ffi::CString::new(to.as_os_str().as_bytes()) {
                    Ok(cstr) => cstr,
                    Err(err) => {
                        writeln!(stderr(), "Invalid to path: {err}").ok();
                        std::process::exit(1);
                    },
                };

                match fig_install::macos::swap(from_cstr, to_cstr) {
                    Ok(_) => {
                        writeln!(stdout(), "success").ok();
                    },
                    Err(err) => {
                        writeln!(stderr(), "Failed to swap files: {err}").ok();
                        std::process::exit(1);
                    },
                }
            },
            #[allow(deprecated)]
            InternalSubcommand::CheckSSH { .. } => {
                std::process::exit(1);
            },
            #[cfg(target_os = "macos")]
            InternalSubcommand::BrewUninstall { zap } => {
                let brew_is_reinstalling = crate::util::is_brew_reinstall().await;

                if brew_is_reinstalling {
                    // If we're reinstalling, we don't want to uninstall
                    return Ok(());
                } else {
                    let url = fig_install::get_uninstall_url(true);
                    fig_util::open_url_async(url).await.ok();
                }

                let components = if zap {
                    // All except the desktop app
                    InstallComponents::all() & !InstallComponents::DESKTOP_APP
                } else {
                    InstallComponents::SHELL_INTEGRATIONS | InstallComponents::SSH | InstallComponents::DAEMON
                };
                fig_install::uninstall(components).await.ok();
            },
            InternalSubcommand::GenerateSSH { remote_username } => {
                let mut should_generate_config = fig_settings::settings::get_bool_or("integrations.ssh.enabled", true);

                for username in ["git", "aur"] {
                    if remote_username == username {
                        should_generate_config = false;
                    }
                }

                let config_path = directories::fig_data_dir()?.join("ssh_inner");

                if should_generate_config {
                    let uuid = uuid::Uuid::new_v4();
                    let exe_path = std::env::current_exe()?;
                    let exe_path = exe_path.to_string_lossy();

                    let config = format!(
                        "# automatically generated by fig\n\
                        # do not edit\n\n\
                        Match all\n  \
                        RemoteForward /var/tmp/fig-parent-{uuid}.socket /var/tmp/fig/${{USER}}/secure.socket\n  \
                        SetEnv LC_FIG_SET_PARENT={uuid} FIG_SET_PARENT={uuid}\n  \
                        StreamLocalBindMask 600\n  \
                        StreamLocalBindUnlink yes\n  \
                        PermitLocalCommand yes\n  \
                        LocalCommand {exe_path} _ ssh-local-command '%r@%n' '{uuid}' 1>&2\n"
                    );

                    std::fs::write(config_path, config)?;
                    writeln!(stdout(), "wrote inner config").ok();
                } else {
                    std::fs::write(config_path, fig_integrations::ssh::SSH_CONFIG_EMPTY)?;
                    writeln!(stdout(), "cleared inner config").ok();
                }
            },
            InternalSubcommand::GhostText { buffer } => {
                let Ok(session_id) = std::env::var("CWTERM_SESSION_ID") else {
                    exit(1);
                };

                let Ok(mut conn) =
                    BufferedUnixStream::connect(fig_util::directories::figterm_socket_path(&session_id)?).await
                else {
                    exit(1);
                };

                let Ok(Some(FigtermResponseMessage {
                    response:
                        Some(fig_proto::figterm::figterm_response_message::Response::GhostTextComplete(
                            fig_proto::figterm::GhostTextCompleteResponse {
                                insert_text: Some(insert_text),
                            },
                        )),
                })) = conn
                    .send_recv_message_timeout(
                        fig_proto::figterm::FigtermRequestMessage {
                            request: Some(fig_proto::figterm::figterm_request_message::Request::GhostTextComplete(
                                fig_proto::figterm::GhostTextCompleteRequest { buffer: buffer.clone() },
                            )),
                        },
                        Duration::from_secs(5),
                    )
                    .await
                else {
                    exit(1);
                };

                writeln!(stdout(), "{buffer}{insert_text}").ok();
            },
            InternalSubcommand::GhostTextAccept { buffer, suggestion } => {
                fig_telemetry::send_ghost_text_actioned(true, buffer.len(), suggestion.len())
                    .await
                    .ok();
            },
        }

        Ok(())
    }
}

pub fn get_shell() {
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
            return;
        }
    }
    exit(1);
}

pub async fn pre_cmd() {
    let Ok(session_id) = std::env::var("CWTERM_SESSION_ID") else {
        return;
    };

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
}
