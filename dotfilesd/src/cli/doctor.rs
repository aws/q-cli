use semver::Version;
use std::borrow::Cow;
use std::ffi::OsStr;
use std::future::Future;
use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{anyhow, Context, Result};
use crossterm::style::Stylize;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use regex::Regex;
use std::process::Command;
use thiserror::Error;

use super::diagnostics::{dscl_read, get_diagnostics, verify_integration};
use super::issue::get_shell;
use super::util::OSVersion;
use crate::ipc::{connect_timeout, get_socket_path};

use crate::{
    auth::Credentials,
    util::{app_path_from_bundle_id, fig_dir, glob, glob_dir, home_dir, shell::Shell},
};
use async_trait::async_trait;
use tokio;

use crate::proto::local::DiagnosticsResponse;

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
            DoctorError::Error {
                reason,
                info,
                fix: _,
            } => f
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

fn command_fix<A, I>(args: A) -> Option<DoctorFix>
where
    A: IntoIterator<Item = I> + Send,
    I: AsRef<OsStr> + Send + 'static,
{
    let args = args.into_iter().collect::<Vec<_>>();

    Some(Box::new(move || {
        if let (Some(exe), Some(remaining)) = (args.first(), args.get(1..)) {
            if Command::new(exe).args(remaining).status()?.success() {
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

fn app_version(app: impl AsRef<OsStr>) -> Option<Version> {
    let app_path = app_path_from_bundle_id(app)?;
    println!("app_path {}", app_path);
    let output = Command::new("defaults")
        .args([
            "read",
            &format!("{}/Contents/Info.plist", app_path),
            "CFBundleShortVersionString",
        ])
        .output()
        .ok()?;
    let version = String::from_utf8_lossy(&output.stdout);
    Version::parse(&version).ok()
}

fn print_status_result(name: impl AsRef<str>, status: &Result<(), DoctorError>) {
    match status {
        Ok(()) => {
            println!("✅ {}", name.as_ref());
        }
        Err(DoctorError::Warning(msg)) => {
            println!("🟡 {}", msg);
        }
        Err(DoctorError::Error {
            reason,
            info,
            fix: _,
        }) => {
            println!("❌ {}: {}", name.as_ref(), reason);
            for infoline in info {
                println!("  {}", infoline);
            }
        }
    }
}

#[async_trait]
trait DoctorCheck<T = ()>: Sync
where
    T: Sync + Send + Sized,
{
    fn name(&self) -> Cow<'static, str>;

    fn should_check(&self, _: &T) -> bool {
        true
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
        let path = fig_dir().context("~/.fig/bin/fig does not exist")?;
        Ok(check_file_exists(&path)?)
    }
}

struct PathCheck;

#[async_trait]
impl DoctorCheck for PathCheck {
    fn name(&self) -> Cow<'static, str> {
        "Fig in PATH".into()
    }

    async fn check(&self, _: &()) -> Result<(), DoctorError> {
        match std::env::var("PATH").map(|path| path.contains(".fig/bin")) {
            Ok(true) => Ok(()),
            _ => return Err(anyhow!("Path does not contain ~/.fig/bin").into()),
        }
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
            fix: command_fix(vec!["fig", "launch"]),
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
        Ok(check_file_exists(&get_socket_path())?)
    }
}

struct FigtermSocketCheck;

#[async_trait]
impl DoctorCheck for FigtermSocketCheck {
    fn name(&self) -> Cow<'static, str> {
        "Figterm socket exists".into()
    }

    async fn check(&self, _: &()) -> Result<(), DoctorError> {
        let term_session = std::env::var("TERM_SESSION_ID").context("No TERM_SESSION_ID")?;

        let session_id = term_session
            .as_str()
            .split(':')
            .last()
            .ok_or(anyhow!("Invalid TERM_SESSION_ID"))?;

        let socket_path = PathBuf::from("/tmp").join(format!("figterm-{}.socket", session_id));

        check_file_exists(&socket_path)?;

        let conn = match connect_timeout(&socket_path, Duration::from_secs(2)).await {
            Ok(connection) => connection,
            Err(e) => return Err(anyhow!("Socket exists but could not connect: {}", e).into()),
        };

        enable_raw_mode().context("Terminal doesn't support raw mode to verify figterm socket")?;

        let write_handle = tokio::spawn(async move {
            conn.writable().await?;
            tokio::time::sleep(Duration::from_secs(1)).await;
            conn.try_write(b"Testing figterm...\n")
                .map_err(|e| anyhow!(e))
        });

        let mut buffer = String::new();

        std::io::stdin()
            .read_line(&mut buffer)
            .context("Failed reading from terminal")?;

        disable_raw_mode().context("Failed to disable raw mode")?;

        if write_handle.await.is_err() || !buffer.contains("Testing figterm...") {
            return Err(anyhow!("Socket exists but is not writable.").into());
        }

        Ok(())
    }
}

struct DotfileCheck {
    shell: Shell,
    path: PathBuf,
}

#[async_trait]
impl DoctorCheck for DotfileCheck {
    fn name(&self) -> Cow<'static, str> {
        format!("{} contains valid fig hooks", self.path.display()).into()
    }

    async fn check(&self, _: &()) -> Result<(), DoctorError> {
        match self.shell {
            Shell::Fish => {
                // Source order for fish is handled by fish itself.
                if self.path.exists() {
                    return Ok(());
                } else {
                    return Err(DoctorError::Error {
                        reason: format!("{} does not exist", self.path.display()).into(),
                        info: vec![],
                        fix: None,
                    });
                }
            }
            Shell::Zsh | Shell::Bash => {
                // Read file if it exists
                let contents = match read_to_string(&self.path) {
                    Ok(contents) => contents,
                    _ => {
                        return Err(DoctorError::Warning(
                            format!("{} does not exist", self.path.display()).into(),
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

                let first_line = lines.first().copied().unwrap_or_default();
                if first_line.eq("[ -s ~/.fig/shell/pre.sh ] && source ~/.fig/shell/pre.sh") {
                    return Err(DoctorError::Warning(
                        format!("{} has legacy integration", self.path.display()).into(),
                    ));
                }

                let command = format!("dotfiles shell {} pre", self.shell);
                if first_line.ne(&format!("eval \"$({})\"", command)) {
                    let top_lines = lines.get(0..10).map_or(vec![], Vec::from);
                    let top_line_text = top_lines
                        .iter()
                        .enumerate()
                        .map(|(i, x)| format!("{} {}", i + 1, x).into());

                    return Err(DoctorError::Error {

                        reason: format!("Command `{}` is not sourced first in {}", command, self.path.display()).into(),
                        info: vec![
                            "In order for autocomplete to work correctly, Fig's shell integration must be sourced first.".into(),
                            format!("Top of {}:", self.path.display()).into()
                        ].into_iter().chain(top_line_text).collect(),
                        fix: None
                    });
                }

                let last_line = lines.last().copied().unwrap_or_default();
                if last_line.eq("[ -s ~/.fig/fig.sh ] && source ~/.fig/fig.sh") {
                    return Err(DoctorError::Warning(
                        format!("{} has legacy integration", self.path.display()).into(),
                    ));
                }

                let command = format!("dotfiles shell {} post", self.shell);
                if last_line.ne(&format!("eval \"$({})\"", command)) {
                    let n = lines.len();
                    let bottom_lines = lines.get(n - 10..n).map_or(vec![], Vec::from);
                    let bottom_line_text = bottom_lines
                        .iter()
                        .enumerate()
                        .map(|(i, x)| format!("{} {}", n + i + 1, x).into());

                    return Err(DoctorError::Error {
                        reason: format!("Command `{}` is not sourced last in {}", command, self.path.display()).into(),
                        info: vec![
                            "In order for autocomplete to work correctly, Fig's shell integration must be sourced last.".into(),
                            format!("Bottom of {}:", self.path.display()).into()
                        ].into_iter().chain(bottom_line_text).collect(),
                        fix: None
                    });
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
                reason: "Intall script not run".into(),
                info: vec![],
                fix: command_fix(vec!["~/.fig/tools/install_and_upgrade.sh"]),
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
        let exe_path = fig_dir().unwrap().join("bin").join("fig");

        if path != exe_path
            && path != Path::new("/usr/local/bin/.fig/bin/fig")
            && path != Path::new("/usr/local/bin/fig")
        {
            Ok(())
        } else {
            return Err(anyhow!("Fig CLI must be in {}", exe_path.display()).into());
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
                fix: command_fix(vec!["fig", "debug", "prompt-accessibility"]),
            })
        } else {
            Ok(())
        }
    }
}

struct PseudoTerminalPathCheck;
#[async_trait]
impl DoctorCheck<DiagnosticsResponse> for PseudoTerminalPathCheck {
    fn name(&self) -> Cow<'static, str> {
        "PATH and PseudoTerminal PATH match".into()
    }

    async fn check(&self, diagnostics: &DiagnosticsResponse) -> Result<(), DoctorError> {
        let path = std::env::var("PATH").unwrap_or_default();
        if diagnostics.psudoterminal_path.ne(&path) {
            Err(DoctorError::Error {
                reason: "paths do not match".into(),
                info: vec![],
                fix: command_fix(vec!["fig", "app", "set-path"]),
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

    fn should_check(&self, diagnostics: &DiagnosticsResponse) -> bool {
        diagnostics.symlinked == "true"
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

struct ItermIntegrationCheck;

#[async_trait]
impl DoctorCheck for ItermIntegrationCheck {
    fn name(&self) -> Cow<'static, str> {
        "iTerm integration is enabled".into()
    }

    fn should_check(&self, _: &()) -> bool {
        is_installed("com.googlecode.iterm2")
    }

    async fn check(&self, _: &()) -> Result<(), DoctorError> {
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

            let integration_path = home_dir()?.join(
                "Library/Application Support/iTerm2/Scripts/AutoLaunch/fig-iterm-integration.scpt",
            );
            if !integration_path.exists() {
                return Err(anyhow!("fig-iterm-integration.scpt is missing.").into());
            }

            return Err(anyhow!("Unknown error with iTerm integration").into());
        }

        let version =
            app_version("com.googlecode.iterm2").ok_or(anyhow!("Could not get version"))?;
        if version < Version::new(3, 4, 0) {
            return Err(anyhow!(
                "iTerm version is incompatible with Fig. Please update iTerm to latest version"
            )
            .into());
        }
        Ok(())
    }
}

struct ItermBashIntegrationCheck;

#[async_trait]
impl DoctorCheck for ItermBashIntegrationCheck {
    fn name(&self) -> Cow<'static, str> {
        "iTerm bash integration configured".into()
    }

    fn should_check(&self, _: &()) -> bool {
        match home_dir() {
            Ok(home) => home.join(".iterm2_shell_integration.bash").exists(),
            Err(_) => false,
        }
    }

    async fn check(&self, _: &()) -> Result<(), DoctorError> {
        let integration_file = home_dir()?.join(".iterm2_shell_integration.bash");
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
impl DoctorCheck for HyperIntegrationCheck {
    fn name(&self) -> Cow<'static, str> {
        "Hyper integration is enabled".into()
    }

    fn should_check(&self, _: &()) -> bool {
        is_installed("co.zeit.hyper")
    }

    async fn check(&self, _: &()) -> Result<(), DoctorError> {
        let integration = verify_integration("co.zeit.hyper")
            .await
            .context("Could not verify Hyper integration")?;

        if integration != "installed!" {
            // Check ~/.hyper_plugins/local/fig-hyper-integration/index.js exists
            let integration_path =
                home_dir()?.join(".hyper_plugins/local/fig-hyper-integration/index.js");

            if !integration_path.exists() {
                return Err(anyhow!("fig-hyper-integration plugin is missing.").into());
            }

            let config = read_to_string(home_dir()?.join(".hyper.js"))
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

struct VSCodeIntegrationCheck;

#[async_trait]
impl DoctorCheck for VSCodeIntegrationCheck {
    fn name(&self) -> Cow<'static, str> {
        "VSCode integration is enabled".into()
    }

    fn should_check(&self, _: &()) -> bool {
        is_installed("com.microsoft.VSCode")
    }

    async fn check(&self, _: &()) -> Result<(), DoctorError> {
        let integration = verify_integration("com.microsoft.VSCode")
            .await
            .context("Could not verify VSCode integration")?;

        if integration != "installed!" {
            // Check if withfig.fig exists
            let extensions = home_dir()?.join(".vscode").join("extensions");

            let glob_set = glob(&[extensions.join("withfig.fig-").to_string_lossy()]).unwrap();

            let fig_extensions =
                glob_dir(&glob_set, extensions).context("Could not read extensions")?;

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
        if let Ok(creds) = Credentials::load_credentials() {
            if creds.get_access_token().is_some()
                && creds.get_id_token().is_some()
                && creds.get_refresh_token().is_some()
            {
                return Ok(());
            }
        }
        return Err(anyhow!("Not logged in. Run `dotfiles login` to login.").into());
    }
}

async fn run_checks_with_context<T, Fut>(
    header: impl AsRef<str>,
    checks: Vec<&dyn DoctorCheck<T>>,
    get_context: impl Fn() -> Fut,
) -> Result<()>
where
    T: Sync + Send,
    Fut: Future<Output = Result<T>>,
{
    println!("{}", header.as_ref().dark_grey());
    let mut context = get_context().await?;
    for check in checks {
        let name = check.name();
        if !check.should_check(&context) {
            continue;
        }
        let result = check.check(&context).await;
        print_status_result(&name, &result);
        if let Err(DoctorError::Error {
            reason,
            info: _,
            fix,
        }) = result
        {
            if let Some(fixfn) = fix {
                println!("Attempting to fix automatically...");
                if fixfn().is_err() {
                    println!("Failed to fix...");
                } else {
                    println!("Re-running check...");
                    if let Ok(new_context) = get_context().await {
                        context = new_context
                    }
                    let fix_result = check.check(&context).await;
                    print_status_result(&name, &fix_result);
                    match fix_result {
                        Err(DoctorError::Error {
                            reason: _,
                            info: _,
                            fix: _,
                        }) => {}
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
    println!();

    Ok(())
}

async fn get_null_context() -> Result<()> {
    Ok(())
}

async fn run_checks(header: String, checks: Vec<&dyn DoctorCheck>) -> Result<()> {
    run_checks_with_context(header, checks, get_null_context).await
}

// Doctor
pub async fn doctor_cli() -> Result<()> {
    println!("Checking dotfiles...");
    println!();

    let status = async {
        run_checks(
            "Let's make sure Fig is running...".into(),
            vec![
                &FigBinCheck {},
                &PathCheck {},
                &AppRunningCheck {},
                &FigSocketCheck {},
                &FigtermSocketCheck {},
            ],
        )
        .await?;

        run_checks(
            "Let's check your dotfiles...".into(),
            vec![
                // TODO
                &DotfileCheck {
                    shell: Shell::Bash,
                    path: Shell::Bash.get_config_path()?,
                },
                &DotfileCheck {
                    shell: Shell::Zsh,
                    path: Shell::Zsh.get_config_path()?,
                },
            ],
        )
        .await?;

        run_checks(
            "Let's check if your system is compatible...".into(),
            vec![&SystemVersionCheck {}],
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
                &PseudoTerminalPathCheck {},
                &SecureKeyboardCheck {},
                &DotfilesSymlinkedCheck {},
            ],
            get_diagnostics,
        )
        .await?;

        run_checks(
            "Let's check your integrations...".into(),
            vec![
                &ItermIntegrationCheck {},
                &ItermBashIntegrationCheck {},
                &HyperIntegrationCheck {},
                &VSCodeIntegrationCheck {},
            ],
        )
        .await?;

        run_checks(
            "Let's check if you're logged in...".into(),
            vec![&LoginStatusCheck {}],
        )
        .await?;

        anyhow::Ok(())
    };

    if status.await.is_err() {
        println!();
        println!("❌ Doctor found errors. Please fix them and try again.");
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
        println!();
        println!("✅ Everything looks good!");
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
