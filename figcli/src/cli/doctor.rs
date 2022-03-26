use crate::{
    cli::{
        diagnostics::{dscl_read, get_diagnostics, verify_integration},
        util::OSVersion,
    },
    util::{
        app_path_from_bundle_id, get_shell, glob, glob_dir, is_executable_in_path, launch_fig,
        shell::{Shell, ShellFileIntegration},
        terminal::Terminal,
        LaunchOptions,
    },
};

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use crossterm::{
    cursor, execute,
    style::Stylize,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use fig_ipc::{connect_timeout, get_fig_socket_path, send_recv_message};
use fig_proto::{
    daemon::diagnostic_response::{settings_watcher_status, websocket_status},
    local::DiagnosticsResponse,
    FigProtobufEncodable,
};
use fig_telemetry::Source;
use nix::unistd::geteuid;
use regex::Regex;
use semver::Version;
use serde_json::json;
use std::{
    borrow::Cow,
    ffi::OsStr,
    fs::read_to_string,
    future::Future,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};
use thiserror::Error;
use tokio::{
    self,
    io::{AsyncBufReadExt, AsyncWriteExt},
};
use tracing::error;

use spinners::{Spinner, Spinners};

type DoctorFix = Box<dyn FnOnce() -> Result<()> + Send>;

#[derive(Error)]
enum DoctorError {
    #[error("Warning: {0}")]
    Warning(Cow<'static, str>),
    #[error("Error: {reason}")]
    Error {
        reason: Cow<'static, str>,
        info: Vec<Cow<'static, str>>,
        fix: Option<DoctorFix>,
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

impl From<anyhow::Error> for DoctorError {
    fn from(e: anyhow::Error) -> DoctorError {
        DoctorError::Error {
            reason: format!("{}", e).into(),
            info: vec![],
            fix: None,
        }
    }
}

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
                    let spinner =
                        Spinner::new(Spinners::Dots, "Waiting for command to finish...".into());
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
const CHECKMARK: &str = "\x1b[0;32m✔\x1b[0m";
const DOT: &str = "\x1b[0;33m●\x1b[0m";
const CROSS: &str = "\x1b[0;31m✘\x1b[0m";

fn print_status_result(name: impl AsRef<str>, status: &Result<(), DoctorError>) {
    match status {
        Ok(()) => {
            println!("{} {}", CHECKMARK, name.as_ref());
        }
        Err(DoctorError::Warning(msg)) => {
            println!("{} {}", DOT, msg);
        }
        Err(DoctorError::Error { reason, info, .. }) => {
            println!("{} {}: {}", CROSS, name.as_ref(), reason);
            for infoline in info {
                println!("  {}", infoline);
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[allow(clippy::enum_variant_names)]
enum DoctorCheckType {
    NormalCheck,
    SoftCheck,
    NoCheck,
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
        Regex::new(r"[^a-zA-Z0-9]+")
            .unwrap()
            .replace_all(&name, "_")
            .into()
    }

    fn get_type(&self, _: &T) -> DoctorCheckType {
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

struct PathCheck;

#[async_trait]
impl DoctorCheck for PathCheck {
    fn name(&self) -> Cow<'static, str> {
        "PATH contains ~/.local/bin and ~/.fig/bin".into()
    }

    async fn check(&self, _: &()) -> Result<(), DoctorError> {
        match std::env::var("PATH").map(|path| path.contains(".local/bin")) {
            Ok(true) => {}
            _ => return Err(anyhow!("Path does not contain ~/.local/bin").into()),
        }

        match std::env::var("PATH").map(|path| path.contains(".fig/bin")) {
            Ok(true) => {}
            _ => return Err(anyhow!("Path does not contain ~/.fig/bin").into()),
        }

        Ok(())
    }
}

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
        })
    }
}

struct FigSocketCheck;

#[async_trait]
impl DoctorCheck for FigSocketCheck {
    fn name(&self) -> Cow<'static, str> {
        "Fig socket exists".into()
    }

    async fn check(&self, _: &()) -> Result<(), DoctorError> {
        Ok(check_file_exists(&get_fig_socket_path())?)
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

        let socket_path = PathBuf::from("/tmp").join(format!("figterm-{}.socket", term_session));

        check_file_exists(&socket_path)?;

        let mut conn = match connect_timeout(&socket_path, Duration::from_secs(2)).await {
            Ok(connection) => connection,
            Err(e) => return Err(anyhow!("Socket exists but could not connect: {}", e).into()),
        };

        enable_raw_mode().context("Terminal doesn't support raw mode to verify figterm socket")?;

        let write_handle: tokio::task::JoinHandle<Result<(), DoctorError>> =
            tokio::spawn(async move {
                conn.writable().await.map_err(|e| anyhow!("{}", e))?;
                tokio::time::sleep(Duration::from_secs_f32(0.2)).await;

                let message = fig_proto::figterm::FigtermMessage {
                    command: Some(
                        fig_proto::figterm::figterm_message::Command::InsertTextCommand(
                            fig_proto::figterm::InsertTextCommand {
                                insertion: Some("Testing figterm...\n".into()),
                                deletion: None,
                                offset: None,
                                immediate: Some(true),
                            },
                        ),
                    ),
                };

                let fig_message = message.encode_fig_protobuf()?;

                conn.write(&fig_message)
                    .await
                    .map_err(|e| anyhow!("{}", e))?;

                Ok(())
            });

        let mut buffer = String::new();

        let mut stdin = tokio::io::BufReader::new(tokio::io::stdin());

        let timeout =
            tokio::time::timeout(Duration::from_secs_f32(1.2), stdin.read_line(&mut buffer));

        let timeout_result: Result<(), DoctorError> = match timeout.await {
            Ok(Ok(_)) => {
                if buffer.trim() == "Testing figterm..." {
                    Ok(())
                } else {
                    Err(DoctorError::Warning(
                        format!("Figterm socket did not read buffer correctly, make sure not to do any input while doctor is running: {:?}", buffer)
                            .into(),
                    ))
                }
            }
            Ok(Err(err)) => Err(anyhow!("Figterm socket err: {}", err).into()),
            Err(_) => Err(anyhow!("Figterm socket write timed out after 1s").into()),
        };

        disable_raw_mode().context("Failed to disable raw mode")?;

        match write_handle.await {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => return Err(anyhow!("Failed to write to figterm socket: {}", e).into()),
            Err(e) => return Err(anyhow!("Failed to write to figterm socket: {}", e).into()),
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
        let init_system =
            crate::daemon::InitSystem::get_init_system().context("Could not get init system")?;

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
                    reason: format!(
                        "LaunchAgents directory does not exist at {:?}",
                        launch_agents_path
                    )
                    .into(),
                    info: vec![],
                    fix: Some(Box::new(move || {
                        std::fs::create_dir_all(&launch_agents_path)?;
                        crate::daemon::install_daemon()?;
                        std::thread::sleep(std::time::Duration::from_secs(daemon_fix_sleep_sec));
                        Ok(())
                    })),
                });
            }

            // Check the directory is writable
            // I wish `try` was stable :(
            (|| {
                let mut file = std::fs::File::create(&launch_agents_path.join("test.txt"))
                    .context("Could not create test file")?;
                file.write_all(b"test")
                    .context("Could not write to test file")?;
                file.sync_all().context("Could not sync test file")?;
                std::fs::remove_file(&launch_agents_path.join("test.txt"))
                    .context("Could not remove test file")?;
                anyhow::Ok(())
            })()
            .map_err(|err| DoctorError::Error {
                reason: "LaunchAgents directory is not writable".into(),
                info: vec![
                    "Make sure you have write permissions for the LaunchAgents directory".into(),
                    format!("Path: {:?}", launch_agents_path).into(),
                    format!("Error: {}", err).into(),
                ],
                fix: Some(Box::new(move || Ok(()))),
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
                        format!("Daemon status: {}", n).into(),
                        format!("Init system: {:?}", init_system).into(),
                        format!("Error message: {}", error_message.unwrap_or_default()).into(),
                    ],
                    fix: daemon_fix!(),
                })
            }
            None => Err(DoctorError::Error {
                reason: "Daemon is not running".into(),
                info: vec![format!("Init system: {:?}", init_system).into()],
                fix: daemon_fix!(),
            }),
        }?;

        // Get diagnostics from the daemon
        let socket_path = fig_ipc::daemon::get_daemon_socket_path();

        if !socket_path.exists() {
            return Err(DoctorError::Error {
                reason: "Daemon socket does not exist".into(),
                info: vec![],
                fix: daemon_fix!(),
            });
        }

        let mut conn = match connect_timeout(&socket_path, Duration::from_secs(1)).await {
            Ok(connection) => connection,
            Err(_) => {
                return Err(DoctorError::Error {
                    reason: "Daemon socket exists but could not connect".into(),
                    info: vec![format!("Socket path: {}", socket_path.display()).into()],
                    fix: daemon_fix!(),
                });
            }
        };

        let diagnostic_response_result: Result<Option<fig_proto::daemon::DaemonResponse>> =
            send_recv_message(
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
                                });
                            }
                        }

                        if let Some(status) = diagnostics.websocket_status {
                            if status.status() != websocket_status::Status::Ok {
                                return Err(DoctorError::Error {
                                    reason: status
                                        .error
                                        .unwrap_or_else(|| "Daemon websocket error".into())
                                        .into(),
                                    info: vec![],
                                    fix: daemon_fix!(),
                                });
                            }
                        }
                    }
                    #[allow(unreachable_patterns)]
                    _ => {
                        return Err(DoctorError::Error {
                            reason: "Daemon responded with unexpected response type".into(),
                            info: vec![],
                            fix: daemon_fix!(),
                        });
                    }
                },
                None => {
                    return Err(DoctorError::Error {
                        reason: "Daemon responded with no response type".into(),
                        info: vec![],
                        fix: daemon_fix!(),
                    })
                }
            },
            Ok(None) | Err(_) => {
                return Err(DoctorError::Error {
                    reason: "Daemon accepted request but did not respond".into(),
                    info: vec![format!("Socket path: {}", socket_path.display()).into()],
                    fix: daemon_fix!(),
                });
            }
        }

        Ok(())
    }
}

struct DotfileCheck {
    integration: ShellFileIntegration,
}

#[async_trait]
impl DoctorCheck<Option<Shell>> for DotfileCheck {
    fn name(&self) -> Cow<'static, str> {
        format!("{} contains valid fig hooks", self.integration.filename).into()
    }

    fn get_type(&self, current_shell: &Option<Shell>) -> DoctorCheckType {
        if let Some(shell) = current_shell {
            if *shell == self.integration.shell {
                return DoctorCheckType::NormalCheck;
            }
        }

        if is_executable_in_path(&self.integration.shell.to_string()) {
            DoctorCheckType::SoftCheck
        } else {
            DoctorCheckType::NoCheck
        }
    }

    async fn check(&self, _: &Option<Shell>) -> Result<(), DoctorError> {
        let fix_text = format!(
            "Run {} to reinstall shell integrations for {}",
            "fig install --dotfiles".magenta(),
            self.integration.shell
        );
        let path = self.integration.path();
        match self.integration.shell {
            Shell::Fish => {
                // Source order for fish is handled by fish itself.
                if path.exists() {
                    return Ok(());
                } else {
                    let msg = format!("{} does not exist. {}", path.display(), fix_text);
                    return Err(DoctorError::Error {
                        reason: msg.into(),
                        info: vec![],
                        fix: None,
                    });
                }
            }
            Shell::Zsh | Shell::Bash => {
                // Read file if it exists
                let contents = match read_to_string(&path) {
                    Ok(contents) => contents,
                    _ => {
                        return Err(DoctorError::Warning(
                            format!("{} does not exist. {}", path.display(), fix_text).into(),
                        ))
                    }
                };

                let contents: String = Regex::new(r"\s*#.*")
                    .unwrap()
                    .replace_all(&contents, "")
                    .into();

                let lines: Vec<&str> = contents
                    .split('\n')
                    .filter(|line| !(*line).trim().is_empty())
                    .collect();
                let filtered_lines = lines.join("\n");

                let first_line = lines.first().copied().unwrap_or_default();
                if first_line.eq("[ -s ~/.fig/shell/pre.sh ] && source ~/.fig/shell/pre.sh") {
                    return Err(DoctorError::Warning(
                        format!("{} has legacy integration. {}", path.display(), fix_text).into(),
                    ));
                }

                if let Some(pre) = self.integration.pre_integration() {
                    if !pre.get_source_regex(true)?.is_match(&filtered_lines) {
                        let msg = format!(
                            "Pre shell integration not sourced first in {}",
                            path.display()
                        );

                        let top_lines = lines.get(0..lines.len().min(10)).map_or(vec![], Vec::from);
                        let top_line_text = top_lines
                            .iter()
                            .enumerate()
                            .map(|(i, x)| format!("{} {}", i + 1, x).into());

                        let fix_integration = self.integration.clone();
                        return Err(DoctorError::Error {
                            reason: msg.into(),
                            info: vec![
                                "In order for autocomplete to work correctly, Fig's shell integration must be sourced first.".into(),
                                format!("Top of {}:", path.display()).into()
                            ].into_iter().chain(top_line_text).collect(),
                            fix: Some(Box::new(move || {
                                fix_integration.uninstall()?;
                                fix_integration.install(None)?;
                                Ok(())
                            }))
                        });
                    }
                }

                let last_line = lines.last().copied().unwrap_or_default();
                if last_line.eq("[ -s ~/.fig/fig.sh ] && source ~/.fig/fig.sh") {
                    return Err(DoctorError::Warning(
                        format!("{} has legacy integration", path.display()).into(),
                    ));
                }

                if let Some(post) = self.integration.post_integration() {
                    if !post.get_source_regex(true)?.is_match(&filtered_lines) {
                        let msg = format!(
                            "Post shell integration not sourced last in {}",
                            path.display()
                        );

                        let n = lines.len();

                        let bottom_lines =
                            lines.get(n.saturating_sub(10)..n).map_or(vec![], Vec::from);
                        let bottom_line_text = bottom_lines
                            .iter()
                            .enumerate()
                            .map(|(i, x)| format!("{} {}", n + i + 1, x).into());

                        let fix_integration = self.integration.clone();
                        return Err(DoctorError::Error {
                            reason: msg.into(),
                            info: vec![
                                "In order for autocomplete to work correctly, Fig's shell integration must be sourced last.".into(),
                                format!("Bottom of {}:", path.display()).into()
                            ].into_iter().chain(bottom_line_text).collect(),
                            fix: Some(Box::new(move || {
                                fix_integration.uninstall()?;
                                fix_integration.install(None)?;
                                Ok(())
                            }))
                        });
                    }
                }

                Ok(())
            }
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
        let current_shell = get_shell();
        let current_shell_valid = current_shell.as_ref().map(|s| (s, shell_regex.is_match(s)));
        let default_shell = dscl_read("UserShell");
        let default_shell_valid = default_shell.as_ref().map(|s| (s, shell_regex.is_match(s)));
        match (current_shell_valid, default_shell_valid) {
            (Ok((current_shell, false)), _) => {
                return Err(anyhow!("Current shell {} incompatible", current_shell).into())
            }
            (_, Ok((default_shell, false))) => {
                return Err(anyhow!("Default shell {} incompatible", default_shell).into())
            }
            (Err(_), _) => return Err(anyhow!("Could not get current shell").into()),
            (_, Err(_)) => Err(DoctorError::Warning("Could not get default shell".into())),
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
                info: vec![format!(
                    "To fix run: {}",
                    "fig settings autocomplete.disable false".magenta()
                )
                .into()],
                fix: None,
            })
        }
    }
}

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
        } else if path.ends_with("target/debug/fig") || path.ends_with("target/release/fig") {
            Err(DoctorError::Warning(
                "Running debug build in a non-standard location".into(),
            ))
        } else {
            Err(anyhow!(
                "Fig CLI ({}) must be in {}",
                path.display(),
                local_bin_path.display()
            )
            .into())
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
                fix: command_fix(
                    vec!["fig", "debug", "prompt-accessibility"],
                    Duration::from_secs(1),
                ),
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
        let pty_path = fig_settings::state::get_value("pty.path")?
            .and_then(|s| s.as_str().map(str::to_string));

        if path != pty_path.unwrap_or_default() {
            Err(DoctorError::Error {
                reason: "paths do not match".into(),
                info: vec![],
                fix: command_fix(vec!["fig", "app", "set-path"], None),
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

    fn get_type(&self, diagnostics: &DiagnosticsResponse) -> DoctorCheckType {
        if diagnostics.symlinked == "true" {
            DoctorCheckType::NormalCheck
        } else {
            DoctorCheckType::NoCheck
        }
    }

    async fn check(&self, _: &DiagnosticsResponse) -> Result<(), DoctorError> {
        Err(DoctorError::Warning(
			"It looks like your dotfiles are symlinked. If you need to make modifications, make sure they're made in the right place.".into()
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

        let mut info = vec![format!(
            "Secure keyboard process is {}",
            diagnostics.securekeyboard_path
        )
        .into()];

        if is_installed("com.bitwarden.desktop") {
            let version = app_version("com.bitwarden.desktop");
            match version {
                Some(version) => {
                    if version <= Version::new(1, 27, 0) {
                        return Err(DoctorError::Error {
                            reason: "Secure keyboard input is on".into(),
                            info: vec![
                                "Bitwarden may be enabling secure keyboard entry even when not focused.".into(),
                                "This was fixed in version 1.28.0. See https://github.com/bitwarden/desktop/issues/991 for details.".into(),
                                "To fix: upgrade Bitwarden to the latest version".into()
                            ],
                            fix: None
                        });
                    }
                }
                None => {
                    info.insert(0, "Could not get Bitwarden version".into());
                }
            }
        }

        Err(DoctorError::Error {
            reason: "Secure keyboard input is on".into(),
            info,
            fix: None,
        })
    }
}

struct ItermIntegrationCheck {}

#[async_trait]
impl DoctorCheck<Option<Terminal>> for ItermIntegrationCheck {
    fn name(&self) -> Cow<'static, str> {
        "iTerm integration is enabled".into()
    }

    fn get_type(&self, current_terminal: &Option<Terminal>) -> DoctorCheckType {
        if !is_installed(Terminal::Iterm.to_bundle_id()) {
            return DoctorCheckType::NoCheck;
        }

        if matches!(current_terminal.to_owned(), Some(Terminal::Iterm)) {
            DoctorCheckType::NormalCheck
        } else {
            DoctorCheckType::SoftCheck
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
                        return Err(anyhow!("iTerm API server is not enabled.").into());
                    }
                }
                Err(_) => {
                    return Err(anyhow!("Could not get iTerm API status").into());
                }
            }

            let integration_path = fig_directories::home_dir().context("Could not get home dir")?.join(
                "Library/Application Support/iTerm2/Scripts/AutoLaunch/fig-iterm-integration.scpt",
            );
            if !integration_path.exists() {
                return Err(anyhow!("fig-iterm-integration.scpt is missing.").into());
            }

            return Err(anyhow!("Unknown error with iTerm integration").into());
        }

        if let Some(version) = app_version("com.googlecode.iterm2") {
            if version < Version::new(3, 4, 0) {
                return Err(anyhow!(
                    "iTerm version is incompatible with Fig. Please update iTerm to latest version"
                )
                .into());
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

    fn get_type(&self, current_terminal: &Option<Terminal>) -> DoctorCheckType {
        match fig_directories::home_dir() {
            Some(home) => {
                if !home.join(".iterm2_shell_integration.bash").exists() {
                    return DoctorCheckType::NoCheck;
                }
            }
            None => return DoctorCheckType::NoCheck,
        }

        if matches!(current_terminal.to_owned(), Some(Terminal::Iterm)) {
            DoctorCheckType::NormalCheck
        } else {
            DoctorCheckType::SoftCheck
        }
    }

    async fn check(&self, _: &Option<Terminal>) -> Result<(), DoctorError> {
        let integration_file = fig_directories::home_dir()
            .unwrap()
            .join(".iterm2_shell_integration.bash");
        let integration = read_to_string(integration_file)
            .context("Could not read .iterm2_shell_integration.bash")?;

        match Regex::new(r"V(\d*\.\d*\.\d*)").unwrap().captures(&integration) {
            Some(captures) => {
                let version = captures.get(1).unwrap().as_str();
                if Version::new(0, 4, 0) > Version::parse(version).unwrap() {
					return Err(anyhow!(
                        "iTerm Bash Integration is out of date. Please update in iTerm's menu by selecting \"Install Shell Integration\"."
					).into());
                }
                Ok(())
            }
            None => {
				Err(DoctorError::Warning(
                    "iTerm's Bash Integration is installed, but we could not check the version in ~/.iterm2_shell_integration.bash. Integration may be out of date. You can try updating in iTerm's menu by selecting \"Install Shell Integration\"".into()
				))
            }
        }
    }
}

struct HyperIntegrationCheck;
#[async_trait]
impl DoctorCheck<Option<Terminal>> for HyperIntegrationCheck {
    fn name(&self) -> Cow<'static, str> {
        "Hyper integration is enabled".into()
    }

    fn get_type(&self, current_terminal: &Option<Terminal>) -> DoctorCheckType {
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
                return Err(anyhow!("fig-hyper-integration plugin is missing.").into());
            }

            let config = read_to_string(
                fig_directories::home_dir()
                    .context("Could not get home dir")?
                    .join(".hyper.js"),
            )
            .context("Could not read ~/.hyper.js")?;

            if !config.contains("fig-hyper-integration") {
                return Err(anyhow!(
                    "fig-hyper-integration plugin needs to be added to localPlugins!"
                )
                .into());
            }
            return Err(anyhow!("Unknown error with integration!").into());
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
        if !os_version.is_supported() {
            return Err(anyhow!("{} is not supported", os_version).into());
        } else {
            Ok(())
        }
    }
}

struct VSCodeIntegrationCheck {}

#[async_trait]
impl DoctorCheck<Option<Terminal>> for VSCodeIntegrationCheck {
    fn name(&self) -> Cow<'static, str> {
        "VSCode integration is enabled".into()
    }

    fn get_type(&self, current_terminal: &Option<Terminal>) -> DoctorCheckType {
        if !is_installed(Terminal::Vscode.to_bundle_id())
            && !is_installed(Terminal::VSCodeInsiders.to_bundle_id())
        {
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
                return Err(anyhow!("VSCode extension is missing!").into());
            }
            return Err(anyhow!("Unknown error with integration!").into());
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
        match fig_auth::refresh_credentals().await {
            Ok(_) => Ok(()),
            Err(_) => Err(anyhow!("Not logged in. Run `fig login` to login.").into()),
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
        }
    };
    for check in checks {
        let name: String = check.name().into();
        let check_type: DoctorCheckType = check.get_type(&context);
        if matches!(check_type, DoctorCheckType::NoCheck) {
            continue;
        }

        let mut result = check.check(&context).await;

        if !config.strict && matches!(check_type, DoctorCheckType::SoftCheck) {
            if let Err(DoctorError::Error { reason, .. }) = result {
                result = Err(DoctorError::Warning(reason))
            }
        }

        if config.verbose || matches!(result, Err(_)) {
            stop_spinner(spinner.take())?;
            print_status_result(&name, &result);
        }

        if config.verbose {
            continue;
        }

        if let Err(err) = &result {
            match fig_telemetry::SegmentEvent::new("Doctor Error") {
                Ok(mut event) => {
                    if let Err(err) = event.add_default_properties(Source::Cli) {
                        error!(
                            "Could not add default properties to telemetry event: {}",
                            err
                        );
                    }

                    event.add_property("check", check.analytics_event_name());
                    event.add_property("cli_version", env!("CARGO_PKG_VERSION"));

                    match err {
                        DoctorError::Warning(info) | DoctorError::Error { reason: info, .. } => {
                            event.add_property("info", &**info);
                        }
                    }

                    if let Err(err) = event.send_event().await {
                        error!("Could not send telemetry event: {}", err);
                    }
                }
                Err(err) => {
                    error!("Could not send doctor error telemetry: {}", err);
                }
            }
        }

        if let Err(DoctorError::Error { reason, fix, .. }) = result {
            if let Some(fixfn) = fix {
                println!("Attempting to fix automatically...");
                if let Err(e) = fixfn() {
                    println!("Failed to fix: {}", e);
                } else {
                    println!("Re-running check...");
                    println!();
                    if let Ok(new_context) = get_context().await {
                        context = new_context
                    }
                    let fix_result = check.check(&context).await;
                    print_status_result(&name, &fix_result);
                    match fix_result {
                        Err(DoctorError::Error { .. }) => {}
                        _ => {
                            continue;
                        }
                    }
                }
            }
            println!();
            anyhow::bail!(reason);
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
    Ok(Terminal::current_terminal())
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
    if let Some(sp) = spinner {
        sp.stop();
        execute!(
            std::io::stdout(),
            Clear(ClearType::CurrentLine),
            cursor::Show
        )?;
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
    #[cfg(target_family = "unix")]
    {
        if geteuid().is_root() {
            eprintln!(
                "{}",
                "Running doctor as root is not supported.".red().bold()
            );
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

    let all_dotfile_checks: Vec<_> = shell_integrations
        .iter()
        .map(|p| (&*p) as &dyn DoctorCheck<_>)
        .collect();

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
                &PathCheck {},
                &AppRunningCheck {},
                &FigSocketCheck {},
                &DaemonCheck {},
                &FigtermSocketCheck {},
                &InsertionLockCheck {},
                &PseudoTerminalPathCheck {},
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
        .await?;

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

        run_checks_with_context(
            "Let's check your terminal integrations...",
            vec![
                &ItermIntegrationCheck {},
                &ItermBashIntegrationCheck {},
                &HyperIntegrationCheck {},
                &VSCodeIntegrationCheck {},
            ],
            get_terminal_context,
            config,
            &mut spinner,
        )
        .await?;

        anyhow::Ok(())
    };

    let is_error = status.await.is_err();

    stop_spinner(spinner)?;

    if is_error {
        println!();
        println!(
            "{} Doctor found errors. Please fix them and try again.",
            CROSS
        );
        println!();
        println!(
            "If you are not sure how to fix it, please open an issue with {} to let us know!",
            "fig issue".magenta()
        );
        println!(
            "Or, email us at {}!",
            "hello@fig.io".underlined().dark_cyan()
        );
        println!()
    } else {
        // If early exit is disabled, no errors are thrown
        if !config.verbose {
            println!();
            println!("{} Everything looks good!", CHECKMARK);
        }
        println!();
        println!(
            "Fig still not working? Run {} to let us know!",
            "fig issue".magenta()
        );
        println!(
            "Or, email us at {}!",
            "hello@fig.io".underlined().dark_cyan()
        );
        println!()
    }
    Ok(())
}
