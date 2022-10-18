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

use async_trait::async_trait;
use clap::Args;
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
use eyre::{
    ContextCompat,
    Result,
    WrapErr,
};
use fig_daemon::Daemon;
use fig_integrations::shell::{
    ShellExt,
    ShellIntegration,
};
use fig_integrations::Error as InstallationError;
use fig_ipc::{
    BufferedUnixStream,
    SendMessage,
    SendRecvMessage,
};
use fig_proto::daemon::diagnostic_response::{
    settings_watcher_status,
    websocket_status,
};
use fig_proto::local::DiagnosticsResponse;
use fig_telemetry::{
    TrackEventType,
    TrackSource,
};
use fig_util::system_info::SupportLevel;
use fig_util::{
    directories,
    is_fig_desktop_running,
    launch_fig_desktop,
    Shell,
    Terminal,
};
use futures::future::BoxFuture;
use futures::FutureExt;
use regex::Regex;
use semver::Version;
use serde_json::json;
use spinners::{
    Spinner,
    Spinners,
};
use tokio::io::AsyncBufReadExt;

use super::app::restart_fig;
use crate::cli::diagnostics::verify_integration;
use crate::util::{
    app_path_from_bundle_id,
    glob,
    glob_dir,
    is_executable_in_path,
};

#[derive(Debug, Args, PartialEq, Eq)]
pub struct DoctorArgs {
    /// Run all doctor tests, with no fixes
    #[arg(long)]
    verbose: bool,
    /// Error on warnings
    #[arg(long)]
    strict: bool,
}

impl DoctorArgs {
    pub async fn execute(self) -> Result<()> {
        doctor_cli(self.verbose, self.strict).await
    }
}

enum DoctorFix {
    Sync(Box<dyn FnOnce() -> Result<()> + Send>),
    Async(BoxFuture<'static, Result<()>>),
}

enum DoctorError {
    Warning(Cow<'static, str>),
    Error {
        reason: Cow<'static, str>,
        info: Vec<Cow<'static, str>>,
        fix: Option<DoctorFix>,
        error: Option<eyre::Report>,
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

impl From<eyre::Report> for DoctorError {
    fn from(err: eyre::Report) -> Self {
        DoctorError::Error {
            reason: err.to_string().into(),
            info: vec![],
            fix: None,
            error: Some(err),
        }
    }
}

impl From<fig_util::Error> for DoctorError {
    fn from(err: fig_util::Error) -> Self {
        DoctorError::Error {
            reason: err.to_string().into(),
            info: vec![],
            fix: None,
            error: Some(eyre::Report::from(err)),
        }
    }
}

impl From<fig_daemon::Error> for DoctorError {
    fn from(err: fig_daemon::Error) -> Self {
        DoctorError::Error {
            reason: err.to_string().into(),
            info: vec![],
            fix: None,
            error: Some(eyre::Report::from(err)),
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

#[allow(unused_macros)]
macro_rules! doctor_fix {
    ({ reason: $reason:expr,fix: $fix:expr }) => {
        DoctorError::Error {
            reason: format!($reason).into(),
            info: vec![],
            fix: Some(DoctorFix::Sync(Box::new($fix))),
            error: None,
        }
    };
}

macro_rules! doctor_fix_async {
    ({ reason: $reason:expr,fix: $fix:expr }) => {
        DoctorError::Error {
            reason: format!($reason).into(),
            info: vec![],
            fix: Some(DoctorFix::Async(Box::pin($fix))),
            error: None,
        }
    };
}

fn check_file_exists(path: impl AsRef<Path>) -> Result<()> {
    if !path.as_ref().exists() {
        eyre::bail!("No file at path {}", path.as_ref().display())
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

    Some(DoctorFix::Sync(Box::new(move || {
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
        eyre::bail!(
            "Failed to run {:?}",
            args.iter()
                .filter_map(|s| s.as_ref().to_str())
                .collect::<Vec<_>>()
                .join(" ")
        )
    })))
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
        let path = directories::fig_dir().map_err(eyre::Report::from)?;
        Ok(check_file_exists(&path)?)
    }
}

#[cfg(unix)]
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

#[cfg(unix)]
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
        if !is_fig_desktop_running() {
            Err(DoctorError::Error {
                reason: "Fig app is not running".into(),
                info: vec![],
                fix: command_fix(vec!["fig", "launch"], Duration::from_secs(3)),
                error: None,
            })
        } else {
            Ok(())
        }
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
        let fig_socket_path = directories::fig_socket_path().context("No socket path")?;
        let parent = fig_socket_path.parent().map(PathBuf::from);

        if let Some(parent) = parent {
            if !parent.exists() {
                return Err(DoctorError::Error {
                    reason: "Fig socket parent directory does not exist".into(),
                    info: vec![format!("Path: {}", fig_socket_path.display()).into()],
                    fix: Some(DoctorFix::Sync(Box::new(|| {
                        std::fs::create_dir_all(parent)?;
                        Ok(())
                    }))),
                    error: None,
                });
            }
        }

        check_file_exists(directories::fig_socket_path().expect("No home directory")).map_err(|_| {
            doctor_fix_async!({
                reason: "Fig socket missing",
                fix: restart_fig()
            })
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

        #[cfg(target_os = "windows")]
        if let Some(exe) = fig_util::get_parent_process_exe() {
            if exe.ends_with("cmd.exe") {
                return Err(DoctorError::Error {
                    reason: "CMD isn't supported yet, please use Git Bash or WSL in order to use Fig".into(),
                    info: vec![],
                    fix: None,
                    error: None,
                });
            }

            if exe.ends_with("powershell.exe") {
                return Err(DoctorError::Error {
                    reason: "Powershell isn't supported yet, please use Git Bash or WSL in order to use Fig".into(),
                    info: vec![],
                    fix: None,
                    error: None,
                });
            }
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

        match std::env::var("FIG_INTEGRATION_VERSION").as_deref() {
            Ok("9") => {},
            _ => {
                return Err(DoctorError::Error {
                    reason:
                        "This terminal is not running with the latest Fig integration, please restart your terminal"
                            .into(),
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
        "Figterm".into()
    }

    async fn check(&self, _: &()) -> Result<(), DoctorError> {
        // Check that the socket exists
        let term_session = std::env::var("FIGTERM_SESSION_ID").context("No FIGTERM_SESSION_ID")?;
        let socket_path = fig_util::directories::figterm_socket_path(term_session).context("No figterm path")?;

        if let Err(err) = check_file_exists(&socket_path) {
            return Err(DoctorError::Error {
                reason: "Tried to find the socket file, but it wasn't there.".into(),
                info: vec![
                    "Fig uses the /tmp directory for sockets.".into(),
                    "Did you delete files in /tmp? The OS will clear it automatically.".into(),
                    "Try making a new tab or window in your terminal, then run `fig doctor` again.".into(),
                    format!("No file at path: {socket_path:?}").into(),
                ],
                fix: None,
                error: Some(err),
            });
        }

        // Connect to the socket
        let mut conn = match BufferedUnixStream::connect_timeout(&socket_path, Duration::from_secs(2)).await {
            Ok(connection) => connection,
            Err(err) => return Err(doctor_error!("Socket exists but could not connect: {err}")),
        };

        // Try sending an insert event and ensure it inserts what is expected
        enable_raw_mode().context(
            "Your terminal doesn't support raw mode, which is required to verify that the figterm socket works",
        )?;

        let write_handle: tokio::task::JoinHandle<Result<BufferedUnixStream, DoctorError>> = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs_f32(0.2)).await;

            let message = fig_proto::figterm::FigtermRequestMessage {
                request: Some(fig_proto::figterm::figterm_request_message::Request::InsertText(
                    fig_proto::figterm::InsertTextRequest {
                        insertion: Some("Testing figterm...\n".into()),
                        deletion: None,
                        offset: None,
                        immediate: Some(true),
                        insertion_buffer: None,
                        insert_during_command: Some(true),
                    },
                )),
            };

            conn.send_message(message).await.map_err(|err| doctor_error!("{err}"))?;

            Ok(conn)
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
                            "Figterm socket did not read buffer correctly, don't press any keys while the checks are running: {:?}",
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

        let mut conn = match write_handle.await {
            Ok(Ok(conn)) => conn,
            Ok(Err(err)) => return Err(doctor_error!("Failed to write to figterm socket: {err}")),
            Err(err) => return Err(doctor_error!("Failed to write to figterm socket: {err}")),
        };

        timeout_result?;

        // Figterm diagnostics

        let message = fig_proto::figterm::FigtermRequestMessage {
            request: Some(fig_proto::figterm::figterm_request_message::Request::Diagnostics(
                fig_proto::figterm::DiagnosticsRequest {},
            )),
        };

        let response: Result<Option<fig_proto::figterm::FigtermResponseMessage>> = conn
            .send_recv_message_timeout(message, Duration::from_secs(1))
            .await
            .context("Failed to send/recv message");

        match response {
            Ok(Some(figterm_response)) => match figterm_response.response {
                Some(fig_proto::figterm::figterm_response_message::Response::Diagnostics(
                    fig_proto::figterm::DiagnosticsResponse {
                        zsh_autosuggestion_style,
                        fish_suggestion_style,
                        ..
                    },
                )) => {
                    if let Some(style) = zsh_autosuggestion_style {
                        if let Some(fg) = style.fg {
                            if let Some(fig_proto::figterm::term_color::Color::Indexed(i)) = fg.color {
                                if i == 15 {
                                    return Err(doctor_warning!(
                                        "ZSH_AUTOSUGGEST_HIGHLIGHT_STYLE is set to the same style your text, Fig will not be able to detect what you have typed."
                                    ));
                                }
                            }
                        }
                    }

                    if let Some(style) = fish_suggestion_style {
                        if let Some(fg) = style.fg {
                            if let Some(fig_proto::figterm::term_color::Color::Indexed(i)) = fg.color {
                                if i == 15 {
                                    return Err(doctor_warning!(
                                        "The Fish suggestion color is set to the same style your text, Fig will not be able to detect what you have typed."
                                    ));
                                }
                            }
                        }
                    }
                },
                _ => return Err(doctor_error!("Failed to receive expected message from figterm")),
            },
            Ok(None) => return Err(doctor_error!("Received EOF when trying to receive figterm diagnostics")),
            Err(err) => return Err(doctor_error!("Failed to receive figterm diagnostics: {err}")),
        }

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
        let insertion_lock_path = directories::fig_dir()
            .map_err(eyre::Report::from)?
            .join("insertion-lock");

        if insertion_lock_path.exists() {
            return Err(DoctorError::Error {
                reason: "Insertion lock exists".into(),
                info: vec![],
                fix: Some(DoctorFix::Sync(Box::new(move || {
                    std::fs::remove_file(&insertion_lock_path)?;
                    Ok(())
                }))),
                error: None,
            });
        }

        Ok(())
    }
}

macro_rules! daemon_fix {
    () => {
        Some(DoctorFix::Async(
            async move {
                let path = std::env::current_exe()?;
                Daemon::default().install(&path).await?;
                // Sleep for a few seconds to give the daemon time to install and start
                std::thread::sleep(std::time::Duration::from_secs(5));
                Ok(())
            }
            .boxed(),
        ))
    };
}

struct DaemonCheck;

#[async_trait]
impl DoctorCheck for DaemonCheck {
    fn name(&self) -> Cow<'static, str> {
        "Daemon".into()
    }

    async fn check(&self, _: &()) -> Result<(), DoctorError> {
        // Make sure the daemon is running
        Daemon::default().start().await.map_err(|_| DoctorError::Error {
            reason: "Daemon is not running".into(),
            info: vec![],
            fix: daemon_fix!(),
            error: None,
        })?;

        #[cfg(target_os = "macos")]
        {
            use std::io::Write;

            let launch_agents_path = fig_util::directories::home_dir()
                .map_err(eyre::Report::from)?
                .join("Library/LaunchAgents");

            if !launch_agents_path.exists() {
                return Err(DoctorError::Error {
                    reason: format!("LaunchAgents directory does not exist at {:?}", launch_agents_path).into(),
                    info: vec![],
                    fix: Some(DoctorFix::Async(
                        async move {
                            std::fs::create_dir_all(&launch_agents_path)?;
                            let path = std::env::current_exe()?;
                            fig_daemon::Daemon::default().install(&path).await?;
                            std::thread::sleep(std::time::Duration::from_secs(5));
                            Ok(())
                        }
                        .boxed(),
                    )),
                    error: None,
                });
            }

            // Check the directory is writable
            // I wish `try` was stable :(
            (|| -> Result<()> {
                let mut file = std::fs::File::create(&launch_agents_path.join("test.txt"))
                    .context("Could not create test file")?;
                file.write_all(b"test").context("Could not write to test file")?;
                file.sync_all().context("Could not sync test file")?;
                std::fs::remove_file(&launch_agents_path.join("test.txt")).context("Could not remove test file")?;
                Ok(())
            })()
            .map_err(|err| DoctorError::Error {
                reason: "LaunchAgents directory is not writable".into(),
                info: vec![
                    "Make sure you have write permissions for the LaunchAgents directory".into(),
                    format!("Path: {:?}", launch_agents_path).into(),
                    format!("Error: {err}").into(),
                ],
                fix: Some(DoctorFix::Sync(Box::new(move || Ok(())))),
                error: None,
            })?;
        }

        match Daemon::default().status().await? {
            Some(0) => Ok(()),
            Some(n) => {
                let error_message = tokio::fs::read_to_string(
                    &directories::fig_dir()
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
                        format!("Error message: {}", error_message.unwrap_or_default()).into(),
                    ],
                    fix: daemon_fix!(),
                    error: None,
                })
            },
            None => Err(DoctorError::Error {
                reason: "Daemon is not running".into(),
                info: vec![],
                fix: daemon_fix!(),
                error: None,
            }),
        }?;

        Ok(())
    }
}

struct DaemonDiagnosticsCheck;

#[async_trait]
impl DoctorCheck for DaemonDiagnosticsCheck {
    fn name(&self) -> Cow<'static, str> {
        "Daemon diagnostics".into()
    }

    async fn check(&self, _: &()) -> Result<(), DoctorError> {
        let socket_path = directories::daemon_socket_path().unwrap();

        cfg_if::cfg_if! {
            if #[cfg(unix)] {
                let socket_exists = socket_path.exists();
            } else if #[cfg(windows)] {
                let socket_exists = match socket_path.metadata() {
                    Ok(_) => true,
                    // Windows can't query socket file existence
                    // Check against arbitrary error code
                    Err(err) => matches!(err.raw_os_error(), Some(1920)),
                };
            }
        }

        if !socket_exists {
            return Err(DoctorError::Error {
                reason: "Daemon socket does not exist".into(),
                info: vec![],
                fix: daemon_fix!(),
                error: None,
            });
        }

        let mut conn = match BufferedUnixStream::connect_timeout(&socket_path, Duration::from_secs(1)).await {
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

        let diagnostic_response_result: Result<Option<fig_proto::daemon::DaemonResponse>> = conn
            .send_recv_message_timeout(fig_proto::daemon::new_diagnostic_message(), Duration::from_secs(1))
            .await
            .context("Failed to send/recv message");

        match diagnostic_response_result {
            Ok(Some(diagnostic_response)) => match diagnostic_response.response {
                Some(response_type) => match response_type {
                    fig_proto::daemon::daemon_response::Response::Diagnostic(diagnostics) => {
                        if let Some(status) = diagnostics.settings_watcher_status {
                            if status.status() != settings_watcher_status::Status::Ok {
                                return Err(DoctorError::Error {
                                    reason: "Daemon settings watcher error".into(),
                                    info: status.error.map(|e| vec![e.into()]).unwrap_or_default(),
                                    fix: daemon_fix!(),
                                    error: None,
                                });
                            }
                        }

                        if let Some(status) = diagnostics.websocket_status {
                            if status.status() != websocket_status::Status::Ok {
                                return Err(DoctorError::Error {
                                    reason: "Daemon websocket error".into(),
                                    info: status.error.map(|e| vec![e.into()]).unwrap_or_default(),
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
        let path = directories::home_dir()
            .ok()
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
            "fig integrations install dotfiles".magenta(),
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
                    fix: Some(DoctorFix::Sync(Box::new(move || {
                        fix_integration.install(None)?;
                        Ok(())
                    }))),
                    error: None,
                })
            },
            Err(err @ InstallationError::FileDoesNotExist(_)) => {
                let fix_integration = self.integration.clone();
                Err(DoctorError::Error {
                    reason: err.to_string().into(),
                    info: vec![fix_text.into()],
                    fix: Some(DoctorFix::Sync(Box::new(move || {
                        fix_integration.install(None)?;
                        Ok(())
                    }))),
                    error: Some(eyre::Report::new(err)),
                })
            },
            Err(err) => Err(DoctorError::Error {
                reason: err.to_string().into(),
                info: vec![],
                fix: None,
                error: Some(eyre::Report::new(err)),
            }),
        }
    }
}

#[cfg(target_os = "macos")]
pub fn dscl_read(value: impl AsRef<OsStr>) -> Result<String> {
    let username_command = Command::new("id").arg("-un").output().context("Could not get id")?;

    let username: String = String::from_utf8_lossy(&username_command.stdout).trim().into();

    let result = Command::new("dscl")
        .arg(".")
        .arg("-read")
        .arg(format!("/Users/{}", username))
        .arg(value)
        .output()
        .context("Could not read value")?;

    Ok(String::from_utf8_lossy(&result.stdout).trim().into())
}

#[cfg(target_os = "macos")]
struct ShellCompatibilityCheck;

#[cfg(target_os = "macos")]
#[async_trait]
impl DoctorCheck<DiagnosticsResponse> for ShellCompatibilityCheck {
    fn name(&self) -> Cow<'static, str> {
        "Compatible shell".into()
    }

    async fn check(&self, _: &DiagnosticsResponse) -> Result<(), DoctorError> {
        let shell_regex = Regex::new(r"(bash|fish|zsh)").unwrap();

        let current_shell = fig_util::get_parent_process_exe();
        let current_shell_valid = current_shell
            .as_ref()
            .map(|path| path.to_string_lossy().to_string())
            .map(|s| {
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

    async fn check(&self, _diagnostics: &DiagnosticsResponse) -> Result<(), DoctorError> {
        if !fig_settings::settings::get_bool_or("autocomplete.disable", false) {
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
        let fig_bin_path = directories::fig_dir().unwrap().join("bin").join("fig");
        let local_bin_path = directories::home_dir().unwrap().join(".local").join("bin").join("fig");

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

struct AutocompleteActiveCheck;

#[async_trait]
impl DoctorCheck<DiagnosticsResponse> for AutocompleteActiveCheck {
    fn name(&self) -> Cow<'static, str> {
        "Autocomplete is active".into()
    }

    fn get_type(&self, diagnostics: &DiagnosticsResponse, _platform: Platform) -> DoctorCheckType {
        if diagnostics.autocomplete_active.is_some() {
            DoctorCheckType::NormalCheck
        } else {
            DoctorCheckType::NoCheck
        }
    }

    async fn check(&self, diagnostics: &DiagnosticsResponse) -> Result<(), DoctorError> {
        if diagnostics.autocomplete_active() {
            Ok(())
        } else {
            Err(doctor_error!(
                "Autocomplete is currently inactive. Your desktop integration(s) may be broken!"
            ))
        }
    }
}

struct ItermIntegrationCheck;

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

            let integration_path = directories::home_dir()
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
            match directories::home_dir() {
                Ok(home) => {
                    if !home.join(".iterm2_shell_integration.bash").exists() {
                        DoctorCheckType::NoCheck
                    } else if matches!(current_terminal.to_owned(), Some(Terminal::Iterm)) {
                        DoctorCheckType::NormalCheck
                    } else {
                        DoctorCheckType::SoftCheck
                    }
                },
                Err(_) => DoctorCheckType::NoCheck,
            }
        } else {
            DoctorCheckType::NoCheck
        }
    }

    async fn check(&self, _: &Option<Terminal>) -> Result<(), DoctorError> {
        let integration_file = directories::home_dir().unwrap().join(".iterm2_shell_integration.bash");
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
            let integration_path = directories::home_dir()
                .context("Could not get home dir")?
                .join(".hyper_plugins/local/fig-hyper-integration/index.js");

            if !integration_path.exists() {
                return Err(doctor_error!("fig-hyper-integration plugin is missing."));
            }

            let config = read_to_string(
                directories::home_dir()
                    .context("Could not get home dir")?
                    .join(".hyper.js"),
            )
            .context("Could not read ~/.hyper.js")?;

            if !config.contains("fig-hyper-integration") {
                return Err(doctor_error!(
                    "fig-hyper-integration plugin needs to be added to localPlugins!"
                ));
            }
            return Err(doctor_error!("Unknown error with Hyper integration"));
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
        let os_version = fig_util::system_info::os_version().wrap_err("Could not get OS Version")?;
        match os_version.support_level() {
            SupportLevel::Supported => Ok(()),
            SupportLevel::InDevelopment { info } => Err(DoctorError::Warning(
                format!(
                    "Fig's support for {os_version} is in development. It may not work properly on your system.\n{}",
                    info.unwrap_or_default()
                )
                .into(),
            )),
            SupportLevel::Unsupported => Err(doctor_error!("{os_version} is not supported")),
        }
    }
}

struct VSCodeIntegrationCheck;

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
            let mut missing = true;

            for dir in [".vscode", ".vscode-insiders"] {
                // Check if withfig.fig exists
                let extensions = directories::home_dir()
                    .context("Could not get home dir")?
                    .join(dir)
                    .join("extensions");

                let glob_set = glob(&[extensions.join("withfig.fig-").to_string_lossy()]).unwrap();

                let extensions = extensions.as_path();
                if let Ok(fig_extensions) = glob_dir(&glob_set, extensions) {
                    if fig_extensions.is_empty() {
                        missing = false;
                    }
                }
            }

            if missing {
                return Err(doctor_error!("VSCode integration is missing!"));
            }

            return Err(doctor_error!("Unknown error with VSCode integration!"));
        }
        Ok(())
    }
}

#[cfg(target_os = "macos")]
struct ImeStatusCheck;

#[cfg(target_os = "macos")]
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
                info: vec!["Run `fig integrations install input-method` to enable it".into()],
                fix: None,
                error: None,
            })
        }
    }
}

#[cfg(target_os = "linux")]
struct IBusEnvCheck;

#[cfg(target_os = "linux")]
#[async_trait]
impl DoctorCheck for IBusEnvCheck {
    fn name(&self) -> Cow<'static, str> {
        "IBus Env Check".into()
    }

    fn get_type(&self, _: &(), _: Platform) -> DoctorCheckType {
        DoctorCheckType::NormalCheck
    }

    async fn check(&self, _: &()) -> Result<(), DoctorError> {
        let err = |var: &str, val: Option<&str>, expect: &str| {
            Err(DoctorError::Error {
                reason: "IBus environment variable is not set".into(),
                info: vec![
                    "Please restart your DE/WM session, for more details see https://fig.io/user-manual/other/linux"
                        .into(),
                    match val {
                        Some(val) => format!("{var} is '{val}', expected '{expect}'").into(),
                        None => format!("{var} is not set, expected '{expect}'").into(),
                    },
                ],
                fix: None,
                error: None,
            })
        };

        let check_env = |var: &str, expect: &str| match std::env::var(var) {
            Ok(val) if val == expect => Ok(()),
            Ok(val) => err(var, Some(&val), expect),
            Err(_) => err(var, None, expect),
        };

        check_env("GTK_IM_MODULE", "ibus")?;
        check_env("QT_IM_MODULE", "ibus")?;
        check_env("XMODIFIERS", "@im=ibus")?;
        // TODO(grant): Add kitty env when fully supported
        Ok(())
    }
}

#[cfg(target_os = "linux")]
struct IBusCheck;

#[cfg(target_os = "linux")]
#[async_trait]
impl DoctorCheck for IBusCheck {
    fn name(&self) -> Cow<'static, str> {
        "IBus Check".into()
    }

    fn get_type(&self, _: &(), _: Platform) -> DoctorCheckType {
        DoctorCheckType::NormalCheck
    }

    async fn check(&self, _: &()) -> Result<(), DoctorError> {
        use sysinfo::{
            ProcessRefreshKind,
            RefreshKind,
            SystemExt,
        };

        let system = sysinfo::System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()));

        if system.processes_by_exact_name("ibus-daemon").next().is_none() {
            return Err(doctor_fix!({
                reason: "ibus-daemon is not running",
                fix: || {
                    let output = Command::new("ibus-daemon").arg("-drxR").output()?;
                    if !output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        eyre::bail!("ibus-daemon launch failed:\nstdout: {stdout}\nstderr: {stderr}\n");
                    }
                    Ok(())
            }}));
        }

        Ok(())
    }
}

struct DesktopCompatibilityCheck;

#[async_trait]
impl DoctorCheck for DesktopCompatibilityCheck {
    fn name(&self) -> Cow<'static, str> {
        "Desktop Compatibility Check".into()
    }

    fn get_type(&self, _: &(), _: Platform) -> DoctorCheckType {
        DoctorCheckType::NormalCheck
    }

    #[cfg(target_os = "linux")]
    async fn check(&self, _: &()) -> Result<(), DoctorError> {
        use fig_util::system_info::linux::{
            get_desktop_environment,
            get_display_server,
            DesktopEnvironment,
            DisplayServer,
        };

        let (display_server, desktop_environment) = (get_display_server()?, get_desktop_environment()?);

        match (display_server, desktop_environment) {
            (DisplayServer::X11, DesktopEnvironment::Gnome | DesktopEnvironment::Plasma | DesktopEnvironment::I3) => {
                Ok(())
            },
            (DisplayServer::Wayland, DesktopEnvironment::Gnome) => Err(doctor_warning!(
                "Fig's support for GNOME on Wayland is in development. It may not work properly on your system."
            )),
            (display_server, desktop_environment) => Err(doctor_warning!(
                "Unknown desktop configuration {desktop_environment:?} on {display_server:?}"
            )),
        }
    }

    #[cfg(not(target_os = "linux"))]
    async fn check(&self, _: &()) -> Result<(), DoctorError> {
        Ok(())
    }
}

struct WindowsConsoleCheck;

#[async_trait]
impl DoctorCheck for WindowsConsoleCheck {
    fn name(&self) -> Cow<'static, str> {
        "Windows Console Check".into()
    }

    fn get_type(&self, _: &(), _: Platform) -> DoctorCheckType {
        DoctorCheckType::NormalCheck
    }

    async fn check(&self, _: &()) -> Result<(), DoctorError> {
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::io::AsRawHandle;

            use winapi::um::consoleapi::GetConsoleMode;

            let mut mode = 0;
            let stdin_ok = unsafe { GetConsoleMode(std::io::stdin().as_raw_handle() as *mut _, &mut mode) };
            let stdout_ok = unsafe { GetConsoleMode(std::io::stdout().as_raw_handle() as *mut _, &mut mode) };

            if stdin_ok != 1 || stdout_ok != 1 {
                return Err(
                    DoctorError::Error {
                        reason: "Windows Console APIs are not supported in this terminal".into(),
                        info: vec![
                            "Fig's PseudoTerminal only supports the new Windows Console API.".into(),
                            "MinTTY and other TTY implementations may not work properly.".into(),
                            "".into(),
                            "You can try the following fixes to get Fig working with your shell:".into(),
                            "- If using Git for Windows, reinstall and choose \"Use default console window\" instead of MinTTY".into(),
                            "- If using Git for Windows and you really want to use MinTTY, reinstall and check \"Enable experimental support for pseudo consoles\"".into(),
                            "- Use your shell with a different supported terminal emulator like Windows Terminal.".into(),
                            "- Launch your terminal emulator with winpty (e.g. winpty mintty). NOTE: this can lead to some UI bugs.".into()
                        ],
                        fix: None,
                        error: None,
                    }
                );
            }
        }
        Ok(())
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
        match fig_request::auth::get_token().await {
            Ok(_) => Ok(()),
            Err(_) => Err(doctor_error!("Not logged in. Run `fig login` to login.")),
        }
    }
}

struct MissionControlHostCheck;

#[async_trait]
impl DoctorCheck for MissionControlHostCheck {
    fn name(&self) -> Cow<'static, str> {
        "Mission Control is loading from the correct URL".into()
    }

    async fn check(&self, _: &()) -> Result<(), DoctorError> {
        match fig_settings::settings::get_string("developer.mission-control.host")
            .ok()
            .flatten()
        {
            Some(host) => {
                if host.contains("localhost") {
                    Err(DoctorError::Warning(
                        format!(
                            "developer.mission-control.host = {}, delete this setting if Mission Control fails to load",
                            host
                        )
                        .into(),
                    ))
                } else {
                    Ok(())
                }
            },
            None => Ok(()),
        }
    }
}

#[cfg(target_os = "linux")]
struct SandboxCheck;

#[async_trait]
#[cfg(target_os = "linux")]
impl DoctorCheck for SandboxCheck {
    fn name(&self) -> Cow<'static, str> {
        "Fig is not running in a sandbox".into()
    }

    async fn check(&self, _: &()) -> Result<(), DoctorError> {
        use fig_util::system_info::linux::SandboxKind;

        let kind = fig_util::system_info::linux::detect_sandbox();

        match kind {
            SandboxKind::None => Ok(()),
            SandboxKind::Flatpak => Err(doctor_error!("Running Fig under Flatpak is not supported.")),
            SandboxKind::Snap => Err(doctor_error!("Running Fig under Snap is not supported.")),
            SandboxKind::Docker => Err(doctor_warning!(
                "Fig's support for Docker is in development. It may not work properly on your system."
            )),
            SandboxKind::Container(Some(engine)) => Err(doctor_error!(
                "Running Fig under `{engine}` containers is not supported."
            )),
            SandboxKind::Container(None) => Err(doctor_error!(
                "Running Fig under non-docker containers is not supported."
            )),
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
            eyre::bail!(e);
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

            fig_telemetry::emit_track(fig_telemetry::TrackEvent::new(
                TrackEventType::DoctorError,
                TrackSource::Cli,
                env!("CARGO_PKG_VERSION").into(),
                properties,
            ))
            .await
            .ok();
        }

        if let Err(DoctorError::Error { reason, fix, error, .. }) = result {
            if let Some(fixfn) = fix {
                println!("Attempting to fix automatically...");
                if let Err(err) = match fixfn {
                    DoctorFix::Sync(fixfn) => fixfn(),
                    DoctorFix::Async(fixfn) => fixfn.await,
                } {
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
                Some(err) => eyre::bail!(err),
                None => eyre::bail!(reason),
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

    // Set pseudoterminal path first so we avoid the check failing if it is not set
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

    // If user is logged in, try to launch fig
    launch_fig_desktop(true, false).ok();

    let shell_integrations: Vec<_> = [Shell::Bash, Shell::Zsh, Shell::Fish]
        .into_iter()
        .map(|shell| shell.get_shell_integrations())
        .collect::<Result<Vec<_>, fig_integrations::Error>>()?
        .into_iter()
        .flatten()
        .map(|integration| DotfileCheck { integration })
        .collect();

    let mut all_dotfile_checks: Vec<&dyn DoctorCheck<_>> = vec![];
    all_dotfile_checks.extend(shell_integrations.iter().map(|p| p as &dyn DoctorCheck<_>));

    let status: Result<()> = async {
        run_checks_with_context(
            "Let's check your dotfiles...",
            all_dotfile_checks,
            get_shell_context,
            config,
            &mut spinner,
        )
        .await?;

        run_checks(
            "Let's make sure Fig is setup correctly...".into(),
            vec![
                &FigBinCheck,
                #[cfg(unix)]
                &LocalBinPathCheck,
                #[cfg(target_os = "macos")]
                &FigBinPathCheck,
                #[cfg(target_os = "windows")]
                &WindowsConsoleCheck,
                &FigIntegrationsCheck,
            ],
            config,
            &mut spinner,
        )
        .await?;

        if fig_util::manifest::is_full() {
            run_checks(
                "Let's make sure Fig is running...".into(),
                vec![&AppRunningCheck, &FigSocketCheck, &DaemonCheck, &DaemonDiagnosticsCheck],
                config,
                &mut spinner,
            )
            .await?;
        }

        run_checks(
            "Let's see if Fig is in a working state...".into(),
            vec![
                #[cfg(unix)]
                &FigtermSocketCheck,
                &InsertionLockCheck,
                &PseudoTerminalPathCheck,
                &AutocompleteDevModeCheck,
                &PluginDevModeCheck,
                &MissionControlHostCheck,
            ],
            config,
            &mut spinner,
        )
        .await?;

        run_checks(
            "Let's check if your system is compatible...".into(),
            vec![&SystemVersionCheck],
            config,
            &mut spinner,
        )
        .await
        .ok();

        if fig_util::manifest::is_headless() {
            return Ok(());
        }

        #[cfg(target_os = "macos")]
        {
            use super::diagnostics::get_diagnostics;

            run_checks_with_context(
                format!("Let's check {}...", "fig diagnostic".bold()),
                vec![
                    &ShellCompatibilityCheck,
                    &BundlePathCheck,
                    &AutocompleteEnabledCheck,
                    &FigCLIPathCheck,
                    &AccessibilityCheck,
                    &DotfilesSymlinkedCheck,
                ],
                get_diagnostics,
                config,
                &mut spinner,
            )
            .await?;
        }

        #[cfg(target_os = "linux")]
        {
            use super::diagnostics::get_diagnostics;

            if fig_util::manifest::is_full() && !fig_util::system_info::is_remote() {
                run_checks_with_context(
                    format!("Let's check {}...", "fig diagnostic".bold()),
                    vec![&AutocompleteActiveCheck],
                    get_diagnostics,
                    config,
                    &mut spinner,
                )
                .await?;
            }
        }

        run_checks_with_context(
            "Let's check your terminal integrations...",
            vec![
                // &ItermIntegrationCheck,
                &ItermBashIntegrationCheck,
                // TODO(sean) re-enable on macos once IME/terminal integrations are sorted
                #[cfg(not(target_os = "macos"))]
                &HyperIntegrationCheck,
                #[cfg(not(target_os = "macos"))]
                &VSCodeIntegrationCheck,
                #[cfg(target_os = "macos")]
                &ImeStatusCheck,
            ],
            get_terminal_context,
            config,
            &mut spinner,
        )
        .await?;

        #[cfg(target_os = "linux")]
        {
            // Linux desktop checks
            if fig_util::manifest::is_full() && !fig_util::system_info::is_remote() {
                run_checks(
                    "Let's check Linux integrations".into(),
                    vec![
                        &IBusEnvCheck,
                        &IBusCheck,
                        // &DesktopCompatibilityCheck, // we need a better way of getting the data
                        &SandboxCheck,
                    ],
                    config,
                    &mut spinner,
                )
                .await?;
            }
        }

        Ok(())
    }
    .await;

    let is_error = status.is_err();

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
        println!("  Fig still not working? Run {} to let us know!", "fig issue".magenta());
        println!("  Or, email us at {}!", "hello@fig.io".underlined().dark_cyan());
        println!()
    }

    if fig_settings::state::get_bool_or("doctor.prompt-restart-terminal", false) {
        println!(
            "  {}{}",
            "PS. Autocomplete won't work in any existing terminal sessions, ".bold(),
            "only new ones.".bold().italic()
        );
        println!("  (You might want to restart your terminal emulator)");
        fig_settings::state::set_value("doctor.prompt-restart-terminal", false)?;
    }
    Ok(())
}
