use std::borrow::Cow;
use std::ffi::OsStr;
use std::fmt::Display;
use std::fs::read_to_string;
use std::future::Future;
use std::path::{
    Path,
    PathBuf,
};
use std::process::Command;
use std::time::Duration;

use anyhow::{
    bail,
    Context,
    Result,
};
use async_trait::async_trait;
use crossterm::style::Stylize;
use crossterm::terminal::{
    disable_raw_mode,
    enable_raw_mode,
    Clear,
    ClearType,
};
use crossterm::{
    cursor,
    execute,
};
use fig_directories::home_dir;
use fig_integrations::shell::{
    ShellExt,
    ShellIntegration,
};
use fig_integrations::InstallationError;
use fig_ipc::{
    connect_timeout,
    get_fig_socket_path,
    send_recv_message,
};
use fig_proto::daemon::diagnostic_response::{
    settings_watcher_status,
    websocket_status,
};
use fig_proto::local::DiagnosticsResponse;
use fig_proto::FigProtobufEncodable;
use fig_telemetry::{
    TrackEvent,
    TrackSource,
};
use fig_util::{
    get_parent_process_exe,
    Shell,
    Terminal,
};
use regex::Regex;
use semver::Version;
use serde_json::json;
use spinners::{
    Spinner,
    Spinners,
};
use sysinfo::{
    ProcessRefreshKind,
    RefreshKind,
    SystemExt,
};
use tokio::io::{
    AsyncBufReadExt,
    AsyncWriteExt,
};

use crate::cli::diagnostics::{
    dscl_read,
    verify_integration,
};
use crate::util::{
    app_path_from_bundle_id,
    glob,
    glob_dir,
    is_executable_in_path,
    launch_fig,
    LaunchOptions,
    OSVersion,
    SupportLevel,
};

type DoctorFix = Box<dyn FnOnce() -> Result<()> + Send>;

enum DoctorError {
    Warning(Cow<'static, str>),
    Error {
        reason: Cow<'static, str>,
        info: Vec<Cow<'static, str>>,
        fix: Option<DoctorFix>,
        error: Option<anyhow::Error>,
    },
}

impl std::fmt::Debug for DoctorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            DoctorError::Warning(msg) => f.debug_struct("Warning").field("msg", msg).finish(),
            DoctorError::Error { reason, info, .. } => f
                .debug_struct("Error")
                .field("reason", reason)
                .field("info", info)
                .finish(),
        }
    }
}

impl std::fmt::Display for DoctorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DoctorError::Warning(warning) => write!(f, "Warning: {warning}"),
            DoctorError::Error { reason, .. } => write!(f, "Error: {reason}"),
        }
    }
}

impl From<anyhow::Error> for DoctorError {
    fn from(err: anyhow::Error) -> Self {
        DoctorError::Error {
            reason: err.to_string().into(),
            info: vec![],
            fix: None,
            error: Some(err),
        }
    }
}

impl DoctorError {
    fn warning(reason: impl Into<Cow<'static, str>>) -> DoctorError {
        DoctorError::Warning(reason.into())
    }

    fn error(reason: impl Into<Cow<'static, str>>) -> DoctorError {
        DoctorError::Error {
            reason: reason.into(),
            info: vec![],
            fix: None,
            error: None,
        }
    }
}

macro_rules! doctor_warning {
    ($($arg:tt)*) => {{
        DoctorError::warning(format!($($arg)*))
    }}
}

macro_rules! doctor_error {
    ($($arg:tt)*) => {{
        DoctorError::error(format!($($arg)*))
    }}
}

macro_rules! doctor_fix {
    ({ reason: $reason:expr,fix: $fix:expr }) => {
        DoctorError::Error {
            reason: format!($reason).into(),
            info: vec![],
            fix: Some(Box::new($fix)),
            error: None,
        }
    };
}

// impl From<anyhow::Error> for DoctorError {
//    fn from(err: anyhow::Error) -> Self {
//        DoctorError::Error {
//            reason: err.to_string().into(),
//            info: vec![],
//            fix: None,
//            error: Some(Box::new(err)),
//        }
//    }
//}

fn check_file_exists(path: impl AsRef<Path>) -> Result<()> {
    if !path.as_ref().exists() {
        anyhow::bail!("No file at path {}", path.as_ref().display())
    }
    Ok(())
}

fn command_fix<A, I, D>(args: A, sleep_duration: D) -> Option<DoctorFix>
where
    A: IntoIterator<Item = I> + Send,
    I: AsRef<OsStr> + Send + 'static,
    D: Into<Option<Duration>> + Send + 'static,
{
    let args = args.into_iter().collect::<Vec<_>>();

    Some(Box::new(move || {
        if let (Some(exe), Some(remaining)) = (args.first(), args.get(1..)) {
            if Command::new(exe).args(remaining).status()?.success() {
                if let Some(duration) = sleep_duration.into() {
                    let spinner = Spinner::new(Spinners::Dots, "Waiting for command to finish...".into());
                    std::thread::sleep(duration);
                    stop_spinner(Some(spinner)).ok();
                }
                return Ok(());
            }
        }
        anyhow::bail!(
            "Failed to run {:?}",
            args.iter()
                .filter_map(|s| s.as_ref().to_str())
                .collect::<Vec<_>>()
                .join(" ")
        )
    }))
}

fn is_installed(app: impl AsRef<OsStr>) -> bool {
    match app_path_from_bundle_id(app) {
        Some(x) => !x.is_empty(),
        None => false,
    }
}

pub fn app_version(app: impl AsRef<OsStr>) -> Option<Version> {
    let app_path = app_path_from_bundle_id(app)?;
    let output = Command::new("defaults")
        .args([
            "read",
            &format!("{}/Contents/Info.plist", app_path),
            "CFBundleShortVersionString",
        ])
        .output()
        .ok()?;
    let version = String::from_utf8_lossy(&output.stdout);
    Version::parse(version.trim()).ok()
}

static CHECKMARK: &str = "✔";
static DOT: &str = "●";
static CROSS: &str = "✘";

fn print_status_result(name: impl Display, status: &Result<(), DoctorError>, verbose: bool) {
    match status {
        Ok(()) => {
            println!("{} {name}", CHECKMARK.green());
        },
        Err(DoctorError::Warning(msg)) => {
            println!("{} {msg}", DOT.yellow());
        },
        Err(DoctorError::Error {
            reason, info, error, ..
        }) => {
            println!("{} {name}: {reason}", CROSS.red());
            for infoline in info {
                println!("  {infoline}");
            }
            if let Some(error) = error {
                if verbose {
                    println!("  {error:?}")
                }
            }
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
enum DoctorCheckType {
    NormalCheck,
    SoftCheck,
    NoCheck,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(unused)]
enum Platform {
    MacOs,
    Linux,
    Windows,
    Other,
}

fn get_platform() -> Platform {
    match std::env::consts::OS {
        "macos" => Platform::MacOs,
        "linux" => Platform::Linux,
        "windows" => Platform::Windows,
        _ => Platform::Other,
    }
}

#[async_trait]
trait DoctorCheck<T = ()>: Sync
where
    T: Sync + Send + Sized,
{
    // Name should be _static_ across different user's devices. It is used to generate
    // a unique id for the check used in analytics. If name cannot be unique for some reason, you
    // should override analytics_event_name with the unique name to be sent for analytics.
    fn name(&self) -> Cow<'static, str>;

    fn analytics_event_name(&self) -> String {
        let name = self.name().to_ascii_lowercase();
        Regex::new(r"[^a-zA-Z0-9]+").unwrap().replace_all(&name, "_").into()
    }

    fn get_type(&self, _: &T, _platform: Platform) -> DoctorCheckType {
        DoctorCheckType::NormalCheck
    }

    async fn check(&self, context: &T) -> Result<(), DoctorError>;
}

struct FigBinCheck;

#[async_trait]
impl DoctorCheck for FigBinCheck {
    fn name(&self) -> Cow<'static, str> {
        "Fig bin exists".into()
    }

    async fn check(&self, _: &()) -> Result<(), DoctorError> {
        let path = fig_directories::fig_dir().context("~/.fig/bin/fig does not exist")?;
        Ok(check_file_exists(&path)?)
    }
}

macro_rules! path_check {
    ($name:ident, $path:expr) => {
        struct $name;

        #[async_trait]
        impl DoctorCheck for $name {
            fn name(&self) -> Cow<'static, str> {
                concat!("PATH contains ~/", $path).into()
            }

            async fn check(&self, _: &()) -> Result<(), DoctorError> {
                match std::env::var("PATH").map(|path| path.contains($path)) {
                    Ok(true) => Ok(()),
                    _ => return Err(doctor_error!(concat!("Path does not contain ~/", $path))),
                }
            }
        }
    };
}

path_check!(LocalBinPathCheck, ".local/bin");
#[cfg(target_os = "macos")]
path_check!(FigBinPathCheck, ".fig/bin");

struct AppRunningCheck;

#[async_trait]
impl DoctorCheck for AppRunningCheck {
    fn name(&self) -> Cow<'static, str> {
        "Fig is running".into()
    }

    async fn check(&self, _: &()) -> Result<(), DoctorError> {
        let result = Command::new("lsappinfo")
            .arg("info")
            .arg("-app")
            .arg("com.mschrage.fig")
            .output();

        if let Ok(output) = result {
            if !String::from_utf8_lossy(&output.stdout).trim().is_empty() {
                return Ok(());
            }
        }

        Err(DoctorError::Error {
            reason: "Fig app is not running".into(),
            info: vec![],
            fix: command_fix(vec!["fig", "launch"], Duration::from_secs(3)),
            error: None,
        })
    }

    fn get_type(&self, _: &(), platform: Platform) -> DoctorCheckType {
        if platform == Platform::MacOs {
            DoctorCheckType::NormalCheck
        } else {
            DoctorCheckType::NoCheck
        }
    }
}

struct FigSocketCheck;

#[async_trait]
impl DoctorCheck for FigSocketCheck {
    fn name(&self) -> Cow<'static, str> {
        "Fig socket exists".into()
    }

    async fn check(&self, _: &()) -> Result<(), DoctorError> {
        let fig_socket_path = get_fig_socket_path();
        let parent = fig_socket_path.parent().map(PathBuf::from);

        if let Some(parent) = parent {
            if !parent.exists() {
                return Err(DoctorError::Error {
                    reason: "Fig socket parent directory does not exist".into(),
                    info: vec![format!("Path: {}", fig_socket_path.display()).into()],
                    fix: Some(Box::new(|| {
                        std::fs::create_dir_all(parent)?;
                        Ok(())
                    })),
                    error: None,
                });
            }
        }

        Ok(check_file_exists(&get_fig_socket_path())?)
    }

    fn get_type(&self, _: &(), platform: Platform) -> DoctorCheckType {
        if platform == Platform::MacOs {
            DoctorCheckType::NormalCheck
        } else {
            DoctorCheckType::NoCheck
        }
    }
}

struct FigIntegrationsCheck;

#[async_trait]
impl DoctorCheck for FigIntegrationsCheck {
    fn name(&self) -> Cow<'static, str> {
        "Fig Integration".into()
    }

    async fn check(&self, _: &()) -> Result<(), DoctorError> {
        if let Ok("WarpTerminal") = std::env::var("TERM_PROGRAM").as_deref() {
            return Err(DoctorError::Error {
                reason: "WarpTerminal is not supported".into(),
                info: vec![],
                fix: None,
                error: None,
            });
        }

        if std::env::var_os("__PWSH_LOGIN_CHECKED").is_some() {
            return Err(DoctorError::Error {
                reason: "Powershell is not supported".into(),
                info: vec![],
                fix: None,
                error: None,
            });
        }

        if std::env::var_os("INSIDE_EMACS").is_some() {
            return Err(DoctorError::Error {
                reason: "Emacs is not supported".into(),
                info: vec![],
                fix: None,
                error: None,
            });
        }

        if let Ok("com.vandyke.SecureCRT") = std::env::var("__CFBundleIdentifier").as_deref() {
            return Err(DoctorError::Error {
                reason: "SecureCRT is not supported".into(),
                info: vec![],
                fix: None,
                error: None,
            });
        }

        if std::env::var_os("FIG_PTY").is_some() {
            return Err(DoctorError::Error {
                reason: "Fig can not run in the Fig Pty".into(),
                info: vec![],
                fix: None,
                error: None,
            });
        }

        if std::env::var_os("PROCESS_LAUNCHED_BY_FIG").is_some() {
            return Err(DoctorError::Error {
                reason: "Fig can not run in a process launched by Fig".into(),
                info: vec![],
                fix: None,
                error: None,
            });
        }

        // Check that ~/.fig/bin/figterm exists
        // TODO(grant): Check figterm exe exists
        // let figterm_path = fig_directories::fig_dir()
        //    .context("Could not find ~/.fig")?
        //    .join("bin")
        //    .join("figterm");

        // if !figterm_path.exists() {
        //    return Err(DoctorError::Error {
        //        reason: "figterm does not exist".into(),
        //        info: vec![],
        //        fix: None,
        //    });
        //}

        match std::env::var("FIG_TERM").as_deref() {
            Ok("1") => {},
            Ok(_) | Err(_) => {
                return Err(DoctorError::Error {
                    reason: "Figterm is not running".into(),
                    info: vec![
                        format!(
                            "FIG_INTEGRATION_VERSION={:?}",
                            std::env::var_os("FIG_INTEGRATION_VERSION")
                        )
                        .into(),
                    ],
                    fix: None,
                    error: None,
                });
            },
        };

        Ok(())
    }
}

struct FigtermSocketCheck;

#[async_trait]
impl DoctorCheck for FigtermSocketCheck {
    fn name(&self) -> Cow<'static, str> {
        "Figterm socket".into()
    }

    async fn check(&self, _: &()) -> Result<(), DoctorError> {
        let term_session = std::env::var("TERM_SESSION_ID").context("No TERM_SESSION_ID")?;

        let socket_path = PathBuf::from("/tmp").join(format!("figterm-{term_session}.socket"));

        check_file_exists(&socket_path)?;

        let mut conn = match connect_timeout(&socket_path, Duration::from_secs(2)).await {
            Ok(connection) => connection,
            Err(err) => return Err(doctor_error!("Socket exists but could not connect: {err}")),
        };

        enable_raw_mode().context("Terminal doesn't support raw mode to verify figterm socket")?;

        let write_handle: tokio::task::JoinHandle<Result<(), DoctorError>> = tokio::spawn(async move {
            conn.writable().await.map_err(|e| doctor_error!("{e}"))?;
            tokio::time::sleep(Duration::from_secs_f32(0.2)).await;

            let message = fig_proto::figterm::FigtermMessage {
                command: Some(fig_proto::figterm::figterm_message::Command::InsertTextCommand(
                    fig_proto::figterm::InsertTextCommand {
                        insertion: Some("Testing figterm...\n".into()),
                        deletion: None,
                        offset: None,
                        immediate: Some(true),
                        insertion_buffer: None,
                    },
                )),
            };

            let fig_message = message.encode_fig_protobuf()?;

            conn.write(&fig_message).await.map_err(|e| doctor_error!("{e}"))?;

            Ok(())
        });

        let mut buffer = String::new();

        let mut stdin = tokio::io::BufReader::new(tokio::io::stdin());

        let timeout = tokio::time::timeout(Duration::from_secs_f32(1.2), stdin.read_line(&mut buffer));

        let timeout_result: Result<(), DoctorError> = match timeout.await {
            Ok(Ok(_)) => {
                if buffer.trim() == "Testing figterm..." {
                    Ok(())
                } else {
                    Err(DoctorError::Warning(
                        format!(
                            "Figterm socket did not read buffer correctly, make sure not to do any input while doctor \
                             is running: {:?}",
                            buffer
                        )
                        .into(),
                    ))
                }
            },
            Ok(Err(err)) => Err(doctor_error!("Figterm socket err: {}", err)),
            Err(_) => Err(doctor_error!("Figterm socket write timed out after 1s")),
        };

        disable_raw_mode().context("Failed to disable raw mode")?;

        match write_handle.await {
            Ok(Ok(_)) => {},
            Ok(Err(err)) => return Err(doctor_error!("Failed to write to figterm socket: {err}")),
            Err(err) => return Err(doctor_error!("Failed to write to figterm socket: {err}")),
        }

        timeout_result?;

        Ok(())
    }
}

/// Checks that the insertion lock doesn't exist.
struct InsertionLockCheck;

#[async_trait]
impl DoctorCheck for InsertionLockCheck {
    fn name(&self) -> Cow<'static, str> {
        "Insertion lock does not exist".into()
    }

    async fn check(&self, _: &()) -> Result<(), DoctorError> {
        let insetion_lock_path = fig_directories::fig_dir()
            .context("Could not get fig dir")?
            .join("insertion-lock");

        if insetion_lock_path.exists() {
            return Err(DoctorError::Error {
                reason: "Insertion lock exists".into(),
                info: vec![],
                fix: Some(Box::new(move || {
                    std::fs::remove_file(&insetion_lock_path)?;
                    Ok(())
                })),
                error: None,
            });
        }

        Ok(())
    }
}

struct DaemonCheck;

#[async_trait]
impl DoctorCheck for DaemonCheck {
    fn name(&self) -> Cow<'static, str> {
        "Daemon".into()
    }

    async fn check(&self, _: &()) -> Result<(), DoctorError> {
        // Check if the daemon is running
        let init_system = crate::daemon::InitSystem::get_init_system().context("Could not get init system")?;

        let daemon_fix_sleep_sec = 5;

        macro_rules! daemon_fix {
            () => {
                Some(Box::new(move || {
                    crate::daemon::install_daemon()?;
                    // Sleep for a second to give the daemon time to install and start
                    std::thread::sleep(std::time::Duration::from_secs(daemon_fix_sleep_sec));
                    Ok(())
                }))
            };
        }

        #[cfg(target_os = "macos")]
        {
            use std::io::Write;

            let launch_agents_path = fig_directories::home_dir()
                .context("Could not get home dir")?
                .join("Library/LaunchAgents");

            if !launch_agents_path.exists() {
                return Err(DoctorError::Error {
                    reason: format!("LaunchAgents directory does not exist at {:?}", launch_agents_path).into(),
                    info: vec![],
                    fix: Some(Box::new(move || {
                        std::fs::create_dir_all(&launch_agents_path)?;
                        crate::daemon::install_daemon()?;
                        std::thread::sleep(std::time::Duration::from_secs(daemon_fix_sleep_sec));
                        Ok(())
                    })),
                    error: None,
                });
            }

            // Check the directory is writable
            // I wish `try` was stable :(
            (|| {
                let mut file = std::fs::File::create(&launch_agents_path.join("test.txt"))
                    .context("Could not create test file")?;
                file.write_all(b"test").context("Could not write to test file")?;
                file.sync_all().context("Could not sync test file")?;
                std::fs::remove_file(&launch_agents_path.join("test.txt")).context("Could not remove test file")?;
                anyhow::Ok(())
            })()
            .map_err(|err| DoctorError::Error {
                reason: "LaunchAgents directory is not writable".into(),
                info: vec![
                    "Make sure you have write permissions for the LaunchAgents directory".into(),
                    format!("Path: {:?}", launch_agents_path).into(),
                    format!("Error: {err}").into(),
                ],
                fix: Some(Box::new(move || Ok(()))),
                error: None,
            })?;
        }

        match init_system.daemon_status()? {
            Some(0) => Ok(()),
            Some(n) => {
                let error_message = tokio::fs::read_to_string(
                    &fig_directories::fig_dir()
                        .context("Could not get fig dir")?
                        .join("logs")
                        .join("daemon-exit.log"),
                )
                .await
                .ok();

                Err(DoctorError::Error {
                    reason: "Daemon is not running".into(),
                    info: vec![
                        format!("Daemon status: {n}").into(),
                        format!("Init system: {:?}", init_system).into(),
                        format!("Error message: {}", error_message.unwrap_or_default()).into(),
                    ],
                    fix: daemon_fix!(),
                    error: None,
                })
            },
            None => Err(DoctorError::Error {
                reason: "Daemon is not running".into(),
                info: vec![format!("Init system: {:?}", init_system).into()],
                fix: daemon_fix!(),
                error: None,
            }),
        }?;

        // Get diagnostics from the daemon
        let socket_path = fig_ipc::daemon::get_daemon_socket_path();

        if !socket_path.exists() {
            return Err(DoctorError::Error {
                reason: "Daemon socket does not exist".into(),
                info: vec![],
                fix: daemon_fix!(),
                error: None,
            });
        }

        let mut conn = match connect_timeout(&socket_path, Duration::from_secs(1)).await {
            Ok(connection) => connection,
            Err(err) => {
                return Err(DoctorError::Error {
                    reason: "Daemon socket exists but could not connect".into(),
                    info: vec![
                        format!("Socket path: {}", socket_path.display()).into(),
                        format!("{err:?}").into(),
                    ],
                    fix: daemon_fix!(),
                    error: None,
                });
            },
        };

        let diagnostic_response_result: Result<Option<fig_proto::daemon::DaemonResponse>> = send_recv_message(
            &mut conn,
            fig_proto::daemon::new_diagnostic_message(),
            Duration::from_secs(1),
        )
        .await;

        match diagnostic_response_result {
            Ok(Some(diagnostic_response)) => match diagnostic_response.response {
                Some(response_type) => match response_type {
                    fig_proto::daemon::daemon_response::Response::Diagnostic(diagnostics) => {
                        if let Some(status) = diagnostics.settings_watcher_status {
                            if status.status() != settings_watcher_status::Status::Ok {
                                return Err(DoctorError::Error {
                                    reason: status
                                        .error
                                        .unwrap_or_else(|| "Daemon settings watcher error".into())
                                        .into(),
                                    info: vec![],
                                    fix: daemon_fix!(),
                                    error: None,
                                });
                            }
                        }

                        if let Some(status) = diagnostics.websocket_status {
                            if status.status() != websocket_status::Status::Ok {
                                return Err(DoctorError::Error {
                                    reason: status.error.unwrap_or_else(|| "Daemon websocket error".into()).into(),
                                    info: vec![],
                                    fix: daemon_fix!(),
                                    error: None,
                                });
                            }
                        }
                    },
                    #[allow(unreachable_patterns)]
                    _ => {
                        return Err(DoctorError::Error {
                            reason: "Daemon responded with unexpected response type".into(),
                            info: vec![],
                            fix: daemon_fix!(),
                            error: None,
                        });
                    },
                },
                None => {
                    return Err(DoctorError::Error {
                        reason: "Daemon responded with no response type".into(),
                        info: vec![],
                        fix: daemon_fix!(),
                        error: None,
                    });
                },
            },
            Ok(None) | Err(_) => {
                return Err(DoctorError::Error {
                    reason: "Daemon accepted request but did not respond".into(),
                    info: vec![format!("Socket path: {}", socket_path.display()).into()],
                    fix: daemon_fix!(),
                    error: None,
                });
            },
        }

        Ok(())
    }
}

struct DotfileCheck {
    integration: Box<dyn ShellIntegration>,
}

#[async_trait]
impl DoctorCheck<Option<Shell>> for DotfileCheck {
    fn name(&self) -> Cow<'static, str> {
        let path = home_dir()
            .and_then(|home_dir| self.integration.path().strip_prefix(&home_dir).ok().map(PathBuf::from))
            .map(|path| format!("~/{}", path.display()))
            .unwrap_or_else(|| self.integration.path().display().to_string());

        format!("{path} contains valid fig hooks").into()
    }

    fn analytics_event_name(&self) -> String {
        format!("dotfile_check_{}", self.integration.file_name())
    }

    fn get_type(&self, current_shell: &Option<Shell>, _platform: Platform) -> DoctorCheckType {
        if let Some(shell) = current_shell {
            if *shell == self.integration.get_shell() {
                return DoctorCheckType::NormalCheck;
            }
        }

        if is_executable_in_path(&self.integration.get_shell().to_string()) {
            DoctorCheckType::SoftCheck
        } else {
            DoctorCheckType::NoCheck
        }
    }

    async fn check(&self, _: &Option<Shell>) -> Result<(), DoctorError> {
        let fix_text = format!(
            "Run {} to reinstall shell integrations for {}",
            "fig install --dotfiles".magenta(),
            self.integration.get_shell()
        );
        match self.integration.is_installed() {
            Ok(()) => Ok(()),
            Err(InstallationError::LegacyInstallation(msg)) => {
                Err(DoctorError::Warning(format!("{msg} {fix_text}").into()))
            },
            Err(InstallationError::NotInstalled(msg) | InstallationError::ImproperInstallation(msg)) => {
                // Check permissions of the file
                #[cfg(unix)]
                {
                    use nix::unistd::{
                        access,
                        AccessFlags,
                    };

                    let path = self.integration.path();
                    if path.exists() {
                        access(&self.integration.path(), AccessFlags::R_OK | AccessFlags::W_OK).map_err(|_| {
                            DoctorError::Error {
                                reason: format!("{} is not accessible", path.display()).into(),
                                info: vec![format!("Run `sudo chown $USER {}` to fix", path.display()).into()],
                                fix: None,
                                error: None,
                            }
                        })?;
                    }
                }

                let fix_integration = self.integration.clone();
                Err(DoctorError::Error {
                    reason: msg,
                    info: vec![fix_text.into()],
                    fix: Some(Box::new(move || {
                        fix_integration.install(None)?;
                        Ok(())
                    })),
                    error: None,
                })
            },
            Err(err @ InstallationError::FileDoesNotExist(_)) => {
                let fix_integration = self.integration.clone();
                Err(DoctorError::Error {
                    reason: err.to_string().into(),
                    info: vec![fix_text.into()],
                    fix: Some(Box::new(move || {
                        fix_integration.install(None)?;
                        Ok(())
                    })),
                    error: Some(anyhow::Error::new(err)),
                })
            },
        }
    }
}

struct InstallationScriptCheck;

#[async_trait]
impl DoctorCheck<DiagnosticsResponse> for InstallationScriptCheck {
    fn name(&self) -> Cow<'static, str> {
        "Installation script".into()
    }

    async fn check(&self, diagnostics: &DiagnosticsResponse) -> Result<(), DoctorError> {
        if diagnostics.installscript == "true" {
            Ok(())
        } else {
            Err(DoctorError::Error {
                reason: "Install script not run".into(),
                info: vec![],
                fix: command_fix(vec!["fig", "app", "install"], None),
                error: None,
            })
        }
    }
}

struct ShellCompatibilityCheck;

#[async_trait]
impl DoctorCheck<DiagnosticsResponse> for ShellCompatibilityCheck {
    fn name(&self) -> Cow<'static, str> {
        "Compatible shell".into()
    }

    async fn check(&self, _: &DiagnosticsResponse) -> Result<(), DoctorError> {
        let shell_regex = Regex::new(r"(bash|fish|zsh)").unwrap();

        let current_shell = get_parent_process_exe();
        let current_shell_valid = current_shell.as_ref().map(|path| path.to_string_lossy()).map(|s| {
            let is_match = shell_regex.is_match(&s);
            (s, is_match)
        });

        let default_shell = dscl_read("UserShell");
        let default_shell_valid = default_shell.as_ref().map(|s| (s, shell_regex.is_match(s)));

        match (current_shell_valid, default_shell_valid) {
            (Some((current_shell, false)), _) => {
                return Err(doctor_error!("Current shell {current_shell} incompatible"));
            },
            (_, Ok((default_shell, false))) => {
                return Err(doctor_error!("Default shell {default_shell} incompatible"));
            },
            (None, _) => return Err(doctor_error!("Could not get current shell")),
            (_, Err(_)) => Err(doctor_warning!("Could not get default shell")),
            _ => Ok(()),
        }
    }
}

struct BundlePathCheck;

#[async_trait]
impl DoctorCheck<DiagnosticsResponse> for BundlePathCheck {
    fn name(&self) -> Cow<'static, str> {
        "Fig app installed in the right place".into()
    }

    async fn check(&self, diagnostics: &DiagnosticsResponse) -> Result<(), DoctorError> {
        let path = diagnostics.path_to_bundle.clone();
        if path.contains("/Applications/Fig.app") {
            Ok(())
        } else if path.contains("/Build/Products/Debug/fig.app") {
            Err(DoctorError::Warning(
                format!("Running debug build in {}", path.bold()).into(),
            ))
        } else {
            Err(DoctorError::Error {
                reason: format!("Fig app is installed in {}", path.bold()).into(),
                info: vec![
                    "You need to install Fig in /Applications.".into(),
                    "To fix: uninstall, then reinstall Fig.".into(),
                    "Remember to drag Fig into the Applications folder.".into(),
                ],
                fix: None,
                error: None,
            })
        }
    }
}

struct AutocompleteEnabledCheck;

#[async_trait]
impl DoctorCheck<DiagnosticsResponse> for AutocompleteEnabledCheck {
    fn name(&self) -> Cow<'static, str> {
        "Autocomplete is enabled".into()
    }

    async fn check(&self, diagnostics: &DiagnosticsResponse) -> Result<(), DoctorError> {
        if diagnostics.autocomplete {
            Ok(())
        } else {
            Err(DoctorError::Error {
                reason: "Autocomplete disabled.".into(),
                info: vec![format!("To fix run: {}", "fig settings autocomplete.disable false".magenta()).into()],
                fix: None,
                error: None,
            })
        }
    }
}

macro_rules! dev_mode_check {
    ($struct_name:ident, $check_name:expr, $settings_module:ident, $setting_name:expr) => {
        struct $struct_name;

        #[async_trait]
        impl DoctorCheck for $struct_name {
            fn name(&self) -> Cow<'static, str> {
                $check_name.into()
            }

            async fn check(&self, _: &()) -> Result<(), DoctorError> {
                if let Ok(Some(true)) = fig_settings::$settings_module::get_bool($setting_name) {
                    Err(DoctorError::Warning(concat!($setting_name, " is enabled").into()))
                } else {
                    Ok(())
                }
            }
        }
    };
}

dev_mode_check!(
    AutocompleteDevModeCheck,
    "Autocomplete dev mode",
    settings,
    "autocomplete.developerMode"
);

dev_mode_check!(PluginDevModeCheck, "Plugin dev mode", state, "plugin.developerMode");

struct FigCLIPathCheck;

#[async_trait]
impl DoctorCheck<DiagnosticsResponse> for FigCLIPathCheck {
    fn name(&self) -> Cow<'static, str> {
        "Fig CLI path".into()
    }

    async fn check(&self, _: &DiagnosticsResponse) -> Result<(), DoctorError> {
        let path = std::env::current_exe().context("Could not get executable path.")?;
        let fig_bin_path = fig_directories::fig_dir().unwrap().join("bin").join("fig");
        let local_bin_path = fig_directories::home_dir()
            .unwrap()
            .join(".local")
            .join("bin")
            .join("fig");

        if path == fig_bin_path
            || path == local_bin_path
            || path == Path::new("/usr/local/bin/fig")
            || path == Path::new("/opt/homebrew/bin/fig")
        {
            Ok(())
        } else if path.ends_with("target/debug/fig")
            || path.ends_with("target/release/fig")
            || path.ends_with("target/debug/fig_cli")
            || path.ends_with("target/release/fig_cli")
        {
            Err(DoctorError::Warning(
                "Running debug build in a non-standard location".into(),
            ))
        } else {
            Err(doctor_error!(
                "Fig CLI ({}) must be in {}",
                path.display(),
                local_bin_path.display()
            ))
        }
    }
}

struct AccessibilityCheck;

#[async_trait]
impl DoctorCheck<DiagnosticsResponse> for AccessibilityCheck {
    fn name(&self) -> Cow<'static, str> {
        "Accessibility enabled".into()
    }

    async fn check(&self, diagnostics: &DiagnosticsResponse) -> Result<(), DoctorError> {
        if diagnostics.accessibility != "true" {
            Err(DoctorError::Error {
                reason: "Accessibility is disabled".into(),
                info: vec![],
                fix: command_fix(vec!["fig", "debug", "prompt-accessibility"], Duration::from_secs(1)),
                error: None,
            })
        } else {
            Ok(())
        }
    }
}

struct PseudoTerminalPathCheck;
#[async_trait]
impl DoctorCheck for PseudoTerminalPathCheck {
    fn name(&self) -> Cow<'static, str> {
        "PATH and PseudoTerminal PATH match".into()
    }

    async fn check(&self, _: &()) -> Result<(), DoctorError> {
        let path = std::env::var("PATH").unwrap_or_default();
        let pty_path = fig_settings::state::get_value("pty.path")
            .map_err(|e| DoctorError::Error {
                reason: "Could not get PseudoTerminal PATH".into(),
                info: vec![e.to_string().into()],
                fix: None,
                error: None,
            })?
            .and_then(|s| s.as_str().map(str::to_string));

        if path != pty_path.unwrap_or_default() {
            Err(DoctorError::Error {
                reason: "paths do not match".into(),
                info: vec![],
                fix: command_fix(vec!["fig", "app", "set-path"], None),
                error: None,
            })
        } else {
            Ok(())
        }
    }
}

struct DotfilesSymlinkedCheck;

#[async_trait]
impl DoctorCheck<DiagnosticsResponse> for DotfilesSymlinkedCheck {
    fn name(&self) -> Cow<'static, str> {
        "Dotfiles symlinked".into()
    }

    fn get_type(&self, diagnostics: &DiagnosticsResponse, _platform: Platform) -> DoctorCheckType {
        if diagnostics.symlinked == "true" {
            DoctorCheckType::NormalCheck
        } else {
            DoctorCheckType::NoCheck
        }
    }

    async fn check(&self, _: &DiagnosticsResponse) -> Result<(), DoctorError> {
        Err(DoctorError::Warning(
            "It looks like your dotfiles are symlinked. If you need to make modifications, make sure they're made in \
             the right place."
                .into(),
        ))
    }
}

struct SecureKeyboardCheck;

#[async_trait]
impl DoctorCheck<DiagnosticsResponse> for SecureKeyboardCheck {
    fn name(&self) -> Cow<'static, str> {
        "Secure keyboard input disabled".into()
    }

    async fn check(&self, diagnostics: &DiagnosticsResponse) -> Result<(), DoctorError> {
        if diagnostics.securekeyboard == "false" {
            return Ok(());
        }

        let mut info = vec![format!("Secure keyboard process is {}", diagnostics.securekeyboard_path).into()];

        if is_installed("com.bitwarden.desktop") {
            let version = app_version("com.bitwarden.desktop");
            match version {
                Some(version) => {
                    if version <= Version::new(1, 27, 0) {
                        return Err(DoctorError::Error {
                            reason: "Secure keyboard input is on".into(),
                            info: vec![
                                "Bitwarden may be enabling secure keyboard entry even when not focused.".into(),
                                "This was fixed in version 1.28.0. See \
                                 https://github.com/bitwarden/desktop/issues/991 for details."
                                    .into(),
                                "To fix: upgrade Bitwarden to the latest version".into(),
                            ],
                            fix: None,
                            error: None,
                        });
                    }
                },
                None => {
                    info.insert(0, "Could not get Bitwarden version".into());
                },
            }
        }

        Err(DoctorError::Error {
            reason: "Secure keyboard input is on".into(),
            info,
            fix: None,
            error: None,
        })
    }
}

struct ItermIntegrationCheck {}

#[async_trait]
impl DoctorCheck<Option<Terminal>> for ItermIntegrationCheck {
    fn name(&self) -> Cow<'static, str> {
        "iTerm integration is enabled".into()
    }

    fn get_type(&self, current_terminal: &Option<Terminal>, platform: Platform) -> DoctorCheckType {
        if platform == Platform::MacOs {
            if !is_installed(Terminal::Iterm.to_bundle_id()) {
                DoctorCheckType::NoCheck
            } else if matches!(current_terminal.to_owned(), Some(Terminal::Iterm)) {
                DoctorCheckType::NormalCheck
            } else {
                DoctorCheckType::SoftCheck
            }
        } else {
            DoctorCheckType::NoCheck
        }
    }

    async fn check(&self, _: &Option<Terminal>) -> Result<(), DoctorError> {
        // iTerm Integration
        let integration = verify_integration("com.googlecode.iterm2")
            .await
            .context("Could not verify iTerm integration")?;
        if integration != "installed!" {
            let output = Command::new("defaults")
                .args(["read", "com.googlecode.iterm2", "EnableAPIServer"])
                .output();
            match output {
                Ok(output) => {
                    let api_enabled = String::from_utf8_lossy(&output.stdout);
                    if api_enabled.trim() == "0" {
                        return Err(doctor_error!("iTerm API server is not enabled."));
                    }
                },
                Err(_) => {
                    return Err(doctor_error!("Could not get iTerm API status"));
                },
            }

            let integration_path = fig_directories::home_dir()
                .context("Could not get home dir")?
                .join("Library/Application Support/iTerm2/Scripts/AutoLaunch/fig-iterm-integration.scpt");
            if !integration_path.exists() {
                return Err(doctor_error!("fig-iterm-integration.scpt is missing."));
            }

            return Err(doctor_error!("Unknown error with iTerm integration"));
        }

        if let Some(version) = app_version("com.googlecode.iterm2") {
            if version < Version::new(3, 4, 0) {
                return Err(doctor_error!(
                    "iTerm version is incompatible with Fig. Please update iTerm to latest version"
                ));
            }
        }
        Ok(())
    }
}

struct ItermBashIntegrationCheck;

#[async_trait]
impl DoctorCheck<Option<Terminal>> for ItermBashIntegrationCheck {
    fn name(&self) -> Cow<'static, str> {
        "iTerm bash integration configured".into()
    }

    fn get_type(&self, current_terminal: &Option<Terminal>, platform: Platform) -> DoctorCheckType {
        if platform == Platform::MacOs {
            match fig_directories::home_dir() {
                Some(home) => {
                    if !home.join(".iterm2_shell_integration.bash").exists() {
                        DoctorCheckType::NoCheck
                    } else if matches!(current_terminal.to_owned(), Some(Terminal::Iterm)) {
                        DoctorCheckType::NormalCheck
                    } else {
                        DoctorCheckType::SoftCheck
                    }
                },
                None => DoctorCheckType::NoCheck,
            }
        } else {
            DoctorCheckType::NoCheck
        }
    }

    async fn check(&self, _: &Option<Terminal>) -> Result<(), DoctorError> {
        let integration_file = fig_directories::home_dir()
            .unwrap()
            .join(".iterm2_shell_integration.bash");
        let integration = read_to_string(integration_file).context("Could not read .iterm2_shell_integration.bash")?;

        match Regex::new(r"V(\d*\.\d*\.\d*)").unwrap().captures(&integration) {
            Some(captures) => {
                let version = captures.get(1).unwrap().as_str();
                if Version::new(0, 4, 0) > Version::parse(version).unwrap() {
                    return Err(doctor_error!(
                        "iTerm Bash Integration is out of date. Please update in iTerm's menu by selecting \"Install \
                         Shell Integration\"."
                    ));
                }
                Ok(())
            },
            None => Err(doctor_warning!(
                "iTerm's Bash Integration is installed, but we could not check the version in \
                 ~/.iterm2_shell_integration.bash. Integration may be out of date. You can try updating in iTerm's \
                 menu by selecting \"Install Shell Integration\"",
            )),
        }
    }
}

struct HyperIntegrationCheck;
#[async_trait]
impl DoctorCheck<Option<Terminal>> for HyperIntegrationCheck {
    fn name(&self) -> Cow<'static, str> {
        "Hyper integration is enabled".into()
    }

    fn get_type(&self, current_terminal: &Option<Terminal>, _platform: Platform) -> DoctorCheckType {
        if !is_installed(Terminal::Hyper.to_bundle_id()) {
            return DoctorCheckType::NoCheck;
        }

        if matches!(current_terminal.to_owned(), Some(Terminal::Hyper)) {
            DoctorCheckType::NormalCheck
        } else {
            DoctorCheckType::SoftCheck
        }
    }

    async fn check(&self, _: &Option<Terminal>) -> Result<(), DoctorError> {
        let integration = verify_integration("co.zeit.hyper")
            .await
            .context("Could not verify Hyper integration")?;

        if integration != "installed!" {
            // Check ~/.hyper_plugins/local/fig-hyper-integration/index.js exists
            let integration_path = fig_directories::home_dir()
                .context("Could not get home dir")?
                .join(".hyper_plugins/local/fig-hyper-integration/index.js");

            if !integration_path.exists() {
                return Err(doctor_error!("fig-hyper-integration plugin is missing."));
            }

            let config = read_to_string(
                fig_directories::home_dir()
                    .context("Could not get home dir")?
                    .join(".hyper.js"),
            )
            .context("Could not read ~/.hyper.js")?;

            if !config.contains("fig-hyper-integration") {
                return Err(doctor_error!(
                    "fig-hyper-integration plugin needs to be added to localPlugins!"
                ));
            }
            return Err(doctor_error!("Unknown error with integration!"));
        }

        Ok(())
    }
}

struct SystemVersionCheck;

#[async_trait]
impl DoctorCheck for SystemVersionCheck {
    fn name(&self) -> Cow<'static, str> {
        "OS is supported".into()
    }

    async fn check(&self, _: &()) -> Result<(), DoctorError> {
        let os_version = OSVersion::new().context("Could not get OS Version")?;
        match os_version.support_level() {
            SupportLevel::Supported => Ok(()),
            SupportLevel::InDevelopment => Err(DoctorError::Warning(
                format!("Fig's support for {os_version} is in development. It may not work properly on your system.")
                    .into(),
            )),
            SupportLevel::Unsupported => Err(doctor_error!("{os_version} is not supported")),
        }
    }
}

struct VSCodeIntegrationCheck {}

#[async_trait]
impl DoctorCheck<Option<Terminal>> for VSCodeIntegrationCheck {
    fn name(&self) -> Cow<'static, str> {
        "VSCode integration is enabled".into()
    }

    fn get_type(&self, current_terminal: &Option<Terminal>, _platform: Platform) -> DoctorCheckType {
        if !is_installed(Terminal::Vscode.to_bundle_id()) && !is_installed(Terminal::VSCodeInsiders.to_bundle_id()) {
            return DoctorCheckType::NoCheck;
        }

        if matches!(current_terminal.to_owned(), Some(Terminal::Vscode))
            || matches!(current_terminal.to_owned(), Some(Terminal::VSCodeInsiders))
        {
            DoctorCheckType::NormalCheck
        } else {
            DoctorCheckType::SoftCheck
        }
    }

    async fn check(&self, _: &Option<Terminal>) -> Result<(), DoctorError> {
        let integration = verify_integration("com.microsoft.VSCode")
            .await
            .context("Could not verify VSCode integration")?;

        if integration != "installed!" {
            // Check if withfig.fig exists
            let extensions = fig_directories::home_dir()
                .context("Could not get home dir")?
                .join(".vscode")
                .join("extensions");

            let glob_set = glob(&[extensions.join("withfig.fig-").to_string_lossy()]).unwrap();

            let extensions = extensions.as_path();
            let fig_extensions = glob_dir(&glob_set, &extensions).map_err(|err| {
                DoctorError::Warning(
                    format!(
                        "Could not read VSCode extensions in dir {}: {}",
                        extensions.to_string_lossy(),
                        err
                    )
                    .into(),
                )
            })?;

            if fig_extensions.is_empty() {
                return Err(doctor_error!("VSCode extension is missing!"));
            }
            return Err(doctor_error!("Unknown error with integration!"));
        }
        Ok(())
    }
}

struct ImeStatusCheck;

#[async_trait]
impl DoctorCheck<Option<Terminal>> for ImeStatusCheck {
    fn name(&self) -> Cow<'static, str> {
        "Input Method".into()
    }

    fn get_type(&self, current_terminal: &Option<Terminal>, _platform: Platform) -> DoctorCheckType {
        match current_terminal {
            Some(current_terminal) if current_terminal.is_input_dependant() => DoctorCheckType::NormalCheck,
            _ => DoctorCheckType::NoCheck,
        }
    }

    async fn check(&self, _: &Option<Terminal>) -> Result<(), DoctorError> {
        if fig_settings::state::get_bool_or("input-method.enabled", false) {
            Ok(())
        } else {
            Err(DoctorError::Error {
                reason: "Input Method is not enabled".into(),
                info: vec!["Run `fig install --input-method` to enable it".into()],
                fix: None,
                error: None,
            })
        }
    }
}

struct IbusCheck;

#[async_trait]
impl DoctorCheck for IbusCheck {
    fn name(&self) -> Cow<'static, str> {
        "IBus Check".into()
    }

    fn get_type(&self, _: &(), _: Platform) -> DoctorCheckType {
        DoctorCheckType::NormalCheck
    }

    async fn check(&self, _: &()) -> Result<(), DoctorError> {
        let system = sysinfo::System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()));

        if system.processes_by_exact_name("ibus-daemon").next().is_none() {
            return Err(doctor_fix!({
                reason: "ibus-daemon is not running",
                fix: || {
                    let output = Command::new("ibus-daemon").arg("-drxR").output()?;
                    if !output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        bail!("ibus-daemon launch failed:\nstdout: {stdout}\nstderr: {stderr}\n");
                    }
                    Ok(())
            }}));
        }

        let ibus_engine_output = Command::new("ibus")
            .arg("engine")
            .output()
            .map_err(anyhow::Error::new)?;

        let stdout = String::from_utf8_lossy(&ibus_engine_output.stdout);
        if ibus_engine_output.status.success() && "fig" == stdout.trim() {
            Ok(())
        } else {
            Err(doctor_fix!({
                reason: "ibus-daemon engine is not fig",
                fix: || {
                    let output = Command::new("ibus").args(&["engine", "fig"]).output()?;
                    if !output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        bail!("Setting ibus engine to fig failed:\nstdout: {stdout}\nstderr: {stderr}\n");
                    }
                    Ok(())
                }
            }))
        }
    }
}

struct LoginStatusCheck;

#[async_trait]
impl DoctorCheck for LoginStatusCheck {
    fn name(&self) -> Cow<'static, str> {
        "Logged into Fig".into()
    }

    async fn check(&self, _: &()) -> Result<(), DoctorError> {
        // We reload the credentials here because we want to check if the user is logged in
        match fig_auth::refresh_credentals().await {
            Ok(_) => Ok(()),
            Err(_) => Err(doctor_error!("Not logged in. Run `fig login` to login.")),
        }
    }
}

async fn run_checks_with_context<T, Fut>(
    header: impl AsRef<str>,
    checks: Vec<&dyn DoctorCheck<T>>,
    get_context: impl Fn() -> Fut,
    config: CheckConfiguration,
    spinner: &mut Option<Spinner>,
) -> Result<()>
where
    T: Sync + Send,
    Fut: Future<Output = Result<T>>,
{
    if config.verbose {
        println!("{}", header.as_ref().dark_grey());
    }
    let mut context = match get_context().await {
        Ok(c) => c,
        Err(e) => {
            println!("Failed to get context: {:?}", e);
            anyhow::bail!(e);
        },
    };
    for check in checks {
        let name: String = check.name().into();
        let check_type: DoctorCheckType = check.get_type(&context, get_platform());

        if check_type == DoctorCheckType::NoCheck {
            continue;
        }

        let mut result = check.check(&context).await;

        if !config.strict && check_type == DoctorCheckType::SoftCheck {
            if let Err(DoctorError::Error { reason, .. }) = result {
                result = Err(DoctorError::Warning(reason))
            }
        }

        if config.verbose || matches!(result, Err(_)) {
            stop_spinner(spinner.take())?;
            print_status_result(&name, &result, config.verbose);
        }

        if config.verbose {
            continue;
        }

        if let Err(err) = &result {
            let mut properties: Vec<(&str, &str)> = vec![];
            let analytics_event_name = check.analytics_event_name();
            properties.push(("check", &analytics_event_name));
            properties.push(("cli_version", env!("CARGO_PKG_VERSION")));

            match err {
                DoctorError::Warning(info) | DoctorError::Error { reason: info, .. } => {
                    properties.push(("info", &**info));
                },
            }

            fig_telemetry::emit_track(TrackEvent::DoctorError, TrackSource::Cli, properties)
                .await
                .ok();
        }

        if let Err(DoctorError::Error { reason, fix, error, .. }) = result {
            if let Some(fixfn) = fix {
                println!("Attempting to fix automatically...");
                if let Err(err) = fixfn() {
                    println!("Failed to fix: {err}");
                } else {
                    println!("Re-running check...");
                    println!();
                    if let Ok(new_context) = get_context().await {
                        context = new_context
                    }
                    let fix_result = check.check(&context).await;
                    print_status_result(&name, &fix_result, config.verbose);
                    match fix_result {
                        Err(DoctorError::Error { .. }) => {},
                        _ => {
                            continue;
                        },
                    }
                }
            }
            println!();
            match error {
                Some(err) => anyhow::bail!(err),
                None => anyhow::bail!(reason),
            }
        }
    }

    if config.verbose {
        println!();
    }

    Ok(())
}

async fn get_shell_context() -> Result<Option<Shell>> {
    Ok(Shell::current_shell())
}

async fn get_terminal_context() -> Result<Option<Terminal>> {
    Ok(Terminal::parent_terminal())
}

async fn get_null_context() -> Result<()> {
    Ok(())
}

async fn run_checks(
    header: String,
    checks: Vec<&dyn DoctorCheck>,
    config: CheckConfiguration,
    spinner: &mut Option<Spinner>,
) -> Result<()> {
    run_checks_with_context(header, checks, get_null_context, config, spinner).await
}

fn stop_spinner(spinner: Option<Spinner>) -> Result<()> {
    if let Some(mut sp) = spinner {
        sp.stop();
        execute!(std::io::stdout(), Clear(ClearType::CurrentLine), cursor::Show)?;
        println!();
    }

    Ok(())
}

#[derive(Copy, Clone)]
struct CheckConfiguration {
    verbose: bool,
    strict: bool,
}

// Doctor
pub async fn doctor_cli(verbose: bool, strict: bool) -> Result<()> {
    #[cfg(unix)]
    {
        use nix::unistd::geteuid;
        if geteuid().is_root() {
            eprintln!("{}", "Running doctor as root is not supported.".red().bold());
            if !verbose {
                eprintln!(
                    "{}",
                    "If you know what you're doing, run the command again with --verbose.".red()
                );
                std::process::exit(1);
            }
        }
    }

    let config = CheckConfiguration { verbose, strict };

    let mut spinner: Option<Spinner> = None;
    if !config.verbose {
        spinner = Some(Spinner::new(Spinners::Dots, "Running checks...".into()));
        execute!(std::io::stdout(), cursor::Hide)?;

        ctrlc::set_handler(move || {
            execute!(std::io::stdout(), cursor::Show).ok();
            std::process::exit(1);
        })?;
    }

    // Set psudoterminal path first so we avoid the check failing if it is not set
    if let Ok(path) = std::env::var("PATH") {
        fig_settings::state::set_value("pty.path", json!(path)).ok();
    }

    run_checks(
        "Let's check if you're logged in...".into(),
        vec![&LoginStatusCheck {}],
        config,
        &mut spinner,
    )
    .await?;

    // If user is logged in, launch fig.
    launch_fig(LaunchOptions::new().wait_for_activation().verbose()).ok();

    let shell_integrations: Vec<_> = [Shell::Bash, Shell::Zsh, Shell::Fish]
        .into_iter()
        .map(|shell| shell.get_shell_integrations())
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .map(|integration| DotfileCheck { integration })
        .collect();

    let mut all_dotfile_checks: Vec<&dyn DoctorCheck<_>> = vec![];
    all_dotfile_checks.extend(shell_integrations.iter().map(|p| p as &dyn DoctorCheck<_>));

    let status = async {
        run_checks_with_context(
            "Let's check your dotfiles...",
            all_dotfile_checks,
            get_shell_context,
            config,
            &mut spinner,
        )
        .await?;

        run_checks(
            "Let's make sure Fig is running...".into(),
            vec![
                &FigBinCheck {},
                &LocalBinPathCheck {},
                #[cfg(target_os = "macos")]
                &FigBinPathCheck {},
                &FigIntegrationsCheck {},
                &AppRunningCheck {},
                &FigSocketCheck {},
                &DaemonCheck {},
                &FigtermSocketCheck {},
                &InsertionLockCheck {},
                &PseudoTerminalPathCheck {},
                &AutocompleteDevModeCheck {},
                &PluginDevModeCheck {},
            ],
            config,
            &mut spinner,
        )
        .await?;

        run_checks(
            "Let's check if your system is compatible...".into(),
            vec![&SystemVersionCheck {}],
            config,
            &mut spinner,
        )
        .await
        .ok();

        #[cfg(target_os = "macos")]
        {
            use super::diagnostics::get_diagnostics;

            run_checks_with_context(
                format!("Let's check {}...", "fig diagnostic".bold()),
                vec![
                    &InstallationScriptCheck {},
                    &ShellCompatibilityCheck {},
                    &BundlePathCheck {},
                    &AutocompleteEnabledCheck {},
                    &FigCLIPathCheck {},
                    &AccessibilityCheck {},
                    &SecureKeyboardCheck {},
                    &DotfilesSymlinkedCheck {},
                ],
                get_diagnostics,
                config,
                &mut spinner,
            )
            .await?;
        }

        run_checks_with_context(
            "Let's check your terminal integrations...",
            vec![
                &ItermIntegrationCheck {},
                &ItermBashIntegrationCheck {},
                &HyperIntegrationCheck {},
                &VSCodeIntegrationCheck {},
                &ImeStatusCheck {},
            ],
            get_terminal_context,
            config,
            &mut spinner,
        )
        .await?;

        #[cfg(target_os = "linux")]
        {
            run_checks(
                "Let's check Linux integrations".into(),
                vec![&IbusCheck {}],
                config,
                &mut spinner,
            )
            .await?;
        }

        anyhow::Ok(())
    };

    let is_error = status.await.is_err();

    stop_spinner(spinner)?;

    if is_error {
        println!();
        println!("{} Doctor found errors. Please fix them and try again.", CROSS.red());
        println!();
        println!(
            "If you are not sure how to fix it, please open an issue with {} to let us know!",
            "fig issue".magenta()
        );
        println!("Or, email us at {}!", "hello@fig.io".underlined().dark_cyan());
        println!()
    } else {
        // If early exit is disabled, no errors are thrown
        if !config.verbose {
            println!();
            println!("{} Everything looks good!", CHECKMARK.green());
        }
        println!();
        println!("Fig still not working? Run {} to let us know!", "fig issue".magenta());
        println!("Or, email us at {}!", "hello@fig.io".underlined().dark_cyan());
        println!()
    }
    Ok(())
}
