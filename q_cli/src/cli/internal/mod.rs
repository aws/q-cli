mod generate_ssh;
pub mod local_state;
pub mod should_figterm_launch;

use std::fmt::Display;
use std::io::{
    stdout,
    Read,
    Write,
};
use std::path::PathBuf;
use std::process::{
    Command,
    ExitCode,
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
use fig_util::env_var::QTERM_SESSION_ID;
use fig_util::{
    directories,
    CLI_BINARY_NAME,
};
use rand::distributions::{
    Alphanumeric,
    DistString,
};
use sysinfo::System;
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
}

impl From<InstallArgs> for InstallComponents {
    fn from(args: InstallArgs) -> Self {
        let InstallArgs {
            dotfiles, input_method, ..
        } = args;
        if dotfiles || input_method {
            let mut install_components = InstallComponents::empty();
            install_components.set(InstallComponents::SHELL_INTEGRATIONS, dotfiles);
            install_components.set(InstallComponents::INPUT_METHOD, input_method);
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
    /// the Amazon Q integrations.
    PreCmd {
        #[arg(long, allow_hyphen_values = true)]
        alias: Option<String>,
    },
    /// Change the local-state file
    LocalState(local_state::LocalStateArgs),
    /// Callback used for the internal pseudoterminal
    Callback(CallbackArgs),
    /// Install the Amazon Q cli
    Install(InstallArgs),
    /// Uninstall the Amazon Q cli
    Uninstall {
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
    /// - 2 fallback to Q_TERM env
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
    OpenUninstallPage,
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
        #[arg(long)]
        delete_bundle: Option<String>,
    },
    #[cfg(target_os = "macos")]
    SwapFiles {
        from: PathBuf,
        to: PathBuf,
        #[arg(long)]
        not_same_bundle_name: bool,
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
    GenerateSsh(generate_ssh::GenerateSshArgs),
    InlineShellCompletion {
        #[arg(long, allow_hyphen_values = true)]
        buffer: String,
    },
    InlineShellCompletionAccept {
        #[arg(long, allow_hyphen_values = true)]
        buffer: String,
        #[arg(long, allow_hyphen_values = true)]
        suggestion: String,
    },
}

const BUFFER_SIZE: usize = 1024;

impl InternalSubcommand {
    pub async fn execute(self) -> Result<ExitCode> {
        match self {
            InternalSubcommand::Install(args) => {
                let no_confirm = args.no_confirm;
                let force = args.force;
                install_cli(args.into(), no_confirm, force).await?;
            },
            InternalSubcommand::Uninstall {
                dotfiles,
                input_method,
                binary,
                ssh,
            } => {
                let components = if dotfiles || binary || ssh || input_method {
                    let mut uninstall_components = InstallComponents::empty();
                    uninstall_components.set(InstallComponents::SHELL_INTEGRATIONS, dotfiles);
                    uninstall_components.set(InstallComponents::INPUT_METHOD, input_method);
                    uninstall_components.set(InstallComponents::BINARY, binary);
                    uninstall_components.set(InstallComponents::SSH, ssh);
                    uninstall_components
                } else {
                    InstallComponents::all()
                };
                if components.contains(InstallComponents::BINARY) {
                    if option_env!("Q_IS_PACKAGE_MANAGED").is_some() {
                        println!("Please uninstall using your package manager");
                    } else {
                        fig_install::uninstall(InstallComponents::BINARY).await?;
                        println!("\n{}\n", "The binary was successfully uninstalled".bold());
                    }
                }

                let mut components = components;
                components.set(InstallComponents::BINARY, false);
                fig_install::uninstall(components).await?;
            },
            InternalSubcommand::PreCmd { alias } => pre_cmd(alias).await,
            InternalSubcommand::LocalState(local_state) => local_state.execute().await?,
            InternalSubcommand::Callback(CallbackArgs {
                handler_id,
                filename,
                exit_code,
            }) => {
                trace!("handlerId: {handler_id}");

                let (filename, exit_code) = match (filename, exit_code) {
                    (Some(filename), Some(exit_code)) => {
                        trace!("callback specified filepath ({filename}) and exitCode ({exit_code}) to output!");
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
                            trace!("Read {size} bytes\n{}", std::str::from_utf8(&buffer[..size])?);
                        }

                        let filename: String = tmp_path.to_str().context("invalid file path")?.into();
                        trace!("Done reading from stdin!");
                        (filename, -1)
                    },
                };
                let hook = new_callback_hook(&handler_id, &filename, exit_code);

                info!(
                    "Sending 'handlerId: {handler_id}, filename: {filename}, exitcode: {exit_code}' over unix socket!\n"
                );

                match send_hook_to_socket(hook).await {
                    Ok(()) => debug!("Successfully sent hook"),
                    Err(e) => debug!("Couldn't send hook {e}"),
                }
            },
            InternalSubcommand::GetShell => {},
            InternalSubcommand::Hostname => {
                if let Some(hostname) = System::host_name() {
                    if write!(stdout(), "{hostname}").is_ok() {
                        return Ok(ExitCode::SUCCESS);
                    }
                }
                return Ok(ExitCode::FAILURE);
            },
            InternalSubcommand::ShouldFigtermLaunch => return Ok(should_figterm_launch::should_figterm_launch()),
            InternalSubcommand::Event { payload, apps, name } => {
                let hook = new_event_hook(name, payload, apps);
                send_hook_to_socket(hook).await?;
            },
            InternalSubcommand::Ipc {
                app,
                figterm,
                json,
                recv,
            } => {
                let message = fig_proto::FigMessage::json(serde_json::from_str::<serde_json::Value>(&json)?)?;

                let socket = if app {
                    directories::desktop_socket_path().expect("Failed to get socket path")
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
                if !cfg!(target_os = "macos") {
                    bail!("uninstall-for-all-users is only supported on macOS");
                }

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
                        .args(["-u", user, "--", CLI_BINARY_NAME, "_", "open-uninstall-page"])
                        .status()
                        .await
                    {
                        if exit_status.success() {
                            open_page_success = true;
                        }
                    }
                    if let Ok(exit_status) = tokio::process::Command::new("runuser")
                        .args([
                            "-u",
                            user,
                            "--",
                            CLI_BINARY_NAME,
                            "integrations",
                            "uninstall",
                            "--silent",
                            "all",
                        ])
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
                let _ = writeln!(stdout(), "{}", uuid::Uuid::new_v4());
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
                let exit_code = match fig_util::system_info::linux::detect_sandbox() {
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
                };
                return Ok(ExitCode::from(exit_code));
            },
            InternalSubcommand::OpenUninstallPage => {
                let url = fig_install::UNINSTALL_URL;
                if let Err(err) = fig_util::open_url(url) {
                    info!("Failed to open uninstall directly, trying daemon proxy: {err}");

                    if let Err(err) =
                        fig_ipc::local::send_command_to_socket(fig_proto::local::command::Command::OpenBrowser(
                            fig_proto::local::OpenBrowserCommand { url: url.into() },
                        ))
                        .await
                    {
                        info!("Failed to open uninstall via desktop, no more options: {err}");
                        return Ok(ExitCode::FAILURE);
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

                if let Ok(session_id) = std::env::var(QTERM_SESSION_ID) {
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
                    Ok(_) => {
                        return Ok(ExitCode::SUCCESS);
                    },
                    Err(err) => {
                        println!(
                            "{}",
                            serde_json::to_string(&err).expect("InputMethodError should be serializable")
                        );
                        return Ok(ExitCode::FAILURE);
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
            InternalSubcommand::FinishUpdate {
                relaunch_dashboard,
                delete_bundle,
            } => {
                // Wait some time for the previous installation to close
                tokio::time::sleep(Duration::from_millis(100)).await;

                crate::util::quit_fig(false).await.ok();

                tokio::time::sleep(Duration::from_millis(200)).await;

                if let Some(bundle_path) = delete_bundle {
                    let path = std::path::Path::new(&bundle_path);
                    if path.exists() {
                        tokio::fs::remove_dir_all(&path)
                            .await
                            .map_err(|err| tracing::warn!("Failed to remove {path:?}: {err}"))
                            .ok();
                    }

                    tokio::time::sleep(Duration::from_millis(200)).await;
                }

                launch_fig_desktop(LaunchArgs {
                    wait_for_socket: false,
                    open_dashboard: relaunch_dashboard,
                    immediate_update: false,
                    verbose: false,
                })
                .ok();
            },
            #[cfg(target_os = "macos")]
            InternalSubcommand::SwapFiles {
                from,
                to,
                not_same_bundle_name,
            } => {
                use std::io::stderr;
                use std::os::unix::prelude::OsStrExt;

                let from_cstr = match std::ffi::CString::new(from.as_os_str().as_bytes()).context("Invalid from path") {
                    Ok(cstr) => cstr,
                    Err(err) => {
                        writeln!(stderr(), "Invalid from path: {err}").ok();
                        return Ok(ExitCode::FAILURE);
                    },
                };

                let to_cstr = match std::ffi::CString::new(to.as_os_str().as_bytes()) {
                    Ok(cstr) => cstr,
                    Err(err) => {
                        writeln!(stderr(), "Invalid to path: {err}").ok();
                        return Ok(ExitCode::FAILURE);
                    },
                };

                match fig_install::macos::install(from_cstr, to_cstr, !not_same_bundle_name) {
                    Ok(_) => {
                        writeln!(stdout(), "success").ok();
                    },
                    Err(err) => {
                        writeln!(stderr(), "Failed to swap files: {err}").ok();
                        return Ok(ExitCode::FAILURE);
                    },
                }
            },
            #[cfg(target_os = "macos")]
            InternalSubcommand::BrewUninstall { zap } => {
                let brew_is_reinstalling = crate::util::is_brew_reinstall().await;

                if brew_is_reinstalling {
                    // If we're reinstalling, we don't want to uninstall
                    return Ok(ExitCode::SUCCESS);
                } else {
                    fig_util::open_url_async(fig_install::UNINSTALL_URL).await.ok();
                }

                let components = if zap {
                    // All except the desktop app
                    InstallComponents::all() & !InstallComponents::DESKTOP_APP
                } else {
                    InstallComponents::SHELL_INTEGRATIONS | InstallComponents::SSH
                };
                fig_install::uninstall(components).await.ok();
            },
            InternalSubcommand::GenerateSsh(args) => {
                args.execute()?;
            },
            InternalSubcommand::InlineShellCompletion { buffer } => {
                let Ok(session_id) = std::env::var(QTERM_SESSION_ID) else {
                    return Ok(ExitCode::FAILURE);
                };

                let Ok(mut conn) =
                    BufferedUnixStream::connect(fig_util::directories::figterm_socket_path(&session_id)?).await
                else {
                    return Ok(ExitCode::FAILURE);
                };

                let Ok(Some(FigtermResponseMessage {
                    response:
                        Some(fig_proto::figterm::figterm_response_message::Response::InlineShellCompletion(
                            fig_proto::figterm::InlineShellCompletionResponse {
                                insert_text: Some(insert_text),
                            },
                        )),
                })) = conn
                    .send_recv_message_timeout(
                        fig_proto::figterm::FigtermRequestMessage {
                            request: Some(
                                fig_proto::figterm::figterm_request_message::Request::InlineShellCompletion(
                                    fig_proto::figterm::InlineShellCompletionRequest { buffer: buffer.clone() },
                                ),
                            ),
                        },
                        Duration::from_secs(5),
                    )
                    .await
                else {
                    return Ok(ExitCode::FAILURE);
                };

                writeln!(stdout(), "{buffer}{insert_text}").ok();
            },
            InternalSubcommand::InlineShellCompletionAccept { buffer, suggestion } => {
                fig_telemetry::send_inline_shell_completion_actioned(true, buffer.len(), suggestion.len()).await;
            },
        }

        Ok(ExitCode::SUCCESS)
    }
}

pub async fn pre_cmd(alias: Option<String>) {
    let Ok(session_id) = std::env::var(QTERM_SESSION_ID) else {
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
                        update_alias: true,
                        alias,
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

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[derive(Debug, Parser, PartialEq, Eq)]
    pub struct MockCli {
        #[command(subcommand)]
        pub subcommand: InternalSubcommand,
    }

    #[test]
    fn parse_pre_cmd() {
        assert_eq!(MockCli::parse_from(["_", "pre-cmd"]), MockCli {
            subcommand: InternalSubcommand::PreCmd { alias: None }
        });

        let alias = format!("a='{CLI_BINARY_NAME} a'\nrd=rmdir");
        assert_eq!(MockCli::parse_from(["_", "pre-cmd", "--alias", &alias]), MockCli {
            subcommand: InternalSubcommand::PreCmd { alias: Some(alias) }
        });

        let hyphen_alias = "-='cd -'\n...=../..\nga='git add'";
        assert_eq!(
            MockCli::parse_from(["_", "pre-cmd", "--alias", hyphen_alias]),
            MockCli {
                subcommand: InternalSubcommand::PreCmd {
                    alias: Some(hyphen_alias.to_owned())
                }
            }
        );
    }
}
