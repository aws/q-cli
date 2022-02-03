use std::{path::PathBuf, fs::read_to_string, time::Duration};
use std::future::Future;
use semver::Version;

use anyhow::{Context, Result, anyhow};
use regex::Regex;
use std::process::Command;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::style::Stylize;

use tokio;
use super::diagnostics::{get_diagnostics, dscl_read, verify_integration};
use super::util::{get_os_version};
use super::issue::get_shell;
use crate::{auth::Credentials, util::{shell::Shell, fig_dir, home_dir, glob, glob_dir}};
use crate::ipc::{get_socket_path, connect_timeout};
use async_trait::async_trait;

use crate::proto::local::{DiagnosticsResponse};

enum DoctorStatus {
    Success,
    Warning(String),
    Error {
        reason: String,
        info: Vec<String>,
        fix: Option<Box<dyn FnOnce() -> Result<()> + Send>>
    }
}

fn check_file_exists(path: &PathBuf) -> Result<()> {
    if !path.exists() {
        anyhow::bail!("No file at path {}", path.to_string_lossy().to_owned())
    }
    Ok(())
}


fn command_fix(args: Vec<&'static str>) -> Option<Box<dyn FnOnce() -> Result<()> + Send>> {
    let boxed_args = Box::new(args);
    Some(Box::new(move || {
        match (boxed_args.first(), boxed_args.get(1..)) {
            (Some(exe), Some(remaining)) => {
                if Command::new(exe).args(remaining).status()?.success() {
                    return Ok(())
                }
            }
            _ => {}
        }
        anyhow::bail!("Failed to run {}", boxed_args.join(" "))
    }))
}

fn app_path_from_bundle_id(bundle_id: &str) -> Option<String> {
	let installed_apps = Command::new("mdfind")
      .arg("kMDItemCFBundleIdentifier")
      .arg("=")
      .arg(bundle_id).output().ok()?;
  Some(String::from_utf8_lossy(&installed_apps.stdout).trim().to_string())
}

fn is_installed(app: &str) -> bool {
  match app_path_from_bundle_id(app) {
      Some(x) => x.len() > 0,
      None => false,
  }
}

fn app_version(app:&str) -> Option<Version> {
    let app_path = app_path_from_bundle_id(app)?;
    println!("app_path {}", app_path);
    let output = Command::new("defaults")
        .args(["read", &format!("{}/Contents/Info.plist", app_path), "CFBundleShortVersionString"])
        .output();
    match output {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stdout);
            Version::parse(&version).ok()
        }
        Err(_) => None
    }
}

fn print_status_result(name: &str, status: &DoctorStatus) {
    match status {
        DoctorStatus::Success => {
            println!("✅ {}", name);
        }
        DoctorStatus::Warning(msg) => {
            println!("🟡 {}", msg);
        }
        DoctorStatus::Error { reason, info, fix: _ } => {
            println!("❌ {}: {}", name, reason);
            for infoline in info {
                println!("  {}", infoline);
            }
        }
    }
}

#[async_trait]
trait DoctorCheck<T=()>: Sync where T: Sync + Send + Sized {
    fn should_check(&self, _: &T) -> bool { true }
    async fn check(&self, context: &T) -> DoctorStatus {
        match self.check_basic(context).await {
            Ok(None) => DoctorStatus::Success,
            Ok(Some(warning)) => DoctorStatus::Warning(warning),
            Err(reason) => DoctorStatus::Error {
                reason: reason.to_string(),
                info: vec![],
                fix: None
            }
        }
    }
    async fn check_basic(&self, _: &T) -> Result<Option<String>> { Ok(None) }
    fn name(&self) -> String;
}

struct FigBinCheck;
#[async_trait]
impl DoctorCheck for FigBinCheck {
    async fn check_basic(&self, _: &()) -> Result<Option<String>> {
        let path = fig_dir().context("~/.fig/bin/fig does not exist")?;
        check_file_exists(&path).map(|| None)
    }
    fn name(&self) -> String { "Fig bin exists".to_string() }
}

struct PathCheck;
#[async_trait]
impl DoctorCheck for PathCheck {
    fn name(&self) -> String { "Fig in PATH".to_string() }
    async fn check_basic(&self, _: &()) -> Result<Option<String>> {
        match std::env::var("PATH").map(|path| path.contains(".fig/bin")) {
            Ok(true) => Ok(None),
            _ => anyhow::bail!("Path does not contain ~/.fig/bin".to_string()),
        }
    }
}

struct AppRunningCheck;
#[async_trait]
impl DoctorCheck for AppRunningCheck {
    fn name(&self) -> String { "Fig is running".to_string() }
    async fn check(&self, _: &()) -> DoctorStatus {
        let result = Command::new("lsappinfo")
            .arg("info")
            .arg("-app")
            .arg("com.mschrage.fig")
            .output();
        if let Ok(output) = result {
            if String::from_utf8_lossy(&output.stdout).trim().len() > 0 {
                return DoctorStatus::Success;
            }
        }

        DoctorStatus::Error {
            reason: "Fig app is not running".to_string(),
            info: vec![],
            fix: command_fix(vec!["fig", "launch"])
        }
    }
}

struct FigSocketCheck;
#[async_trait]
impl DoctorCheck for FigSocketCheck {
    fn name(&self) -> String { "Fig socket exists".to_string() }
    async fn check_basic(&self, _: &()) -> Result<Option<String>> {
        check_file_exists(&get_socket_path()).map(|| None)
    }
}

struct FigtermSocketCheck;
#[async_trait]
impl DoctorCheck for FigtermSocketCheck {
    fn name(&self) -> String { "Figterm socket exists".to_string() }
    async fn check_basic(&self, _: &()) -> Result<Option<String>> {
        let term_session = std::env::var("TERM_SESSION_ID")
            .context("No TERM_SESSION_ID")?;

        let session_id = term_session.as_str().split(':').last().ok_or(anyhow!("Invalid TERM_SESSION_ID"))?;
        let socket_path = PathBuf::from("/tmp")
            .join(format!("figterm-{}.socket", session_id));

        check_file_exists(&socket_path)?;

        let conn = match connect_timeout(&socket_path, Duration::from_secs(2)).await {
            Ok(connection) => connection,
            Err(e) => anyhow::bail!("Socket exists but could not connect: {}", e.to_string())
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

        disable_raw_mode()?;

        if write_handle.await.is_err() || !buffer.contains("Testing figterm...") {
            anyhow::bail!("Socket exists but is not writable.");
        }

        Ok(None)
    }
}

struct DotfileCheck {
    shell: Shell,
    path: PathBuf,
}
#[async_trait]
impl DoctorCheck for DotfileCheck {
    fn name(&self) -> String {
        format!(
            "{} contains valid fig hooks",
            self.path.to_string_lossy().to_string()
        )
    }
    async fn check(&self, _: &()) -> DoctorStatus {
        let path = self.path.to_string_lossy().to_string();
        match self.shell {
            Shell::Fish => {
                // Source order for fish is handled by fish itself.
                if self.path.exists() {
                    return DoctorStatus::Success
                } else {
                    return DoctorStatus::Error {
                        reason: format!("{} does not exist", path),
                        info: vec![],
                        fix: None,
                    }
                }
            }
            Shell::Zsh | Shell::Bash => {
                // Read file if it exists
                let contents = match read_to_string(&self.path) {
                    Ok(contents) => contents,
                    _ => {
                        return DoctorStatus::Warning(format!("{} does not exist", path))
                    }
                };

                let contents = Regex::new(r"\s*#.*").unwrap().replace_all(&contents, "").to_string();
                let lines: Vec<&str> = contents.split("\n").filter(|line| (*line).trim().len() != 0).collect();

                let first_line = lines.first().map(|x| *x).unwrap_or("");
                if first_line.eq("[ -s ~/.fig/shell/pre.sh ] && source ~/.fig/shell/pre.sh") {
                    return DoctorStatus::Warning(
                        format!("{} has legacy integration", path)
                    );
                }

                let command = format!("dotfiles shell {} pre", self.shell);
                if first_line.ne(&format!("eval \"$({})\"", command)) {
                    let top_lines: Vec<_> = lines.get(0..10)
                        .map_or(vec![], Vec::from)
                        .iter().enumerate()
                        .map(|(i, x)| format!("{} {}", i + 1, x.to_string()))
                        .collect();

                    return DoctorStatus::Error {

                        reason: format!("Command `{}` is not sourced first in {}", command, path),
                        info: vec![
                            "In order for autocomplete to work correctly, Fig's shell integration must be sourced first.".to_string(),
                            format!("Top of {}:", path)
                        ].into_iter().chain(top_lines.into_iter()).collect(),
                        fix: None
                    }
                }

                let last_line = lines.last().map(|x| *x).unwrap_or("");
                if last_line.eq("[ -s ~/.fig/fig.sh ] && source ~/.fig/fig.sh") {
                    return DoctorStatus::Warning(
                        format!("{} has legacy integration", path)
                    );
                }

                let command = format!("dotfiles shell {} post", self.shell);
                if last_line.ne(&format!("eval \"$({})\"", command)) {
                    let n = lines.len();
                    let bottom_lines: Vec<_> = lines.get(n-10..n)
                        .map_or(vec![], Vec::from)
                        .iter().enumerate()
                        .map(|(i, x)| format!("{} {}", n + i + 1, x.to_string()))
                        .collect();

                    return DoctorStatus::Error {

                        reason: format!("Command `{}` is not sourced last in {}", command, path),
                        info: vec![
                            "In order for autocomplete to work correctly, Fig's shell integration must be sourced last.".to_string(),
                            format!("Bottom of {}:", path)
                        ].into_iter().chain(bottom_lines.into_iter()).collect(),
                        fix: None
                    }
                }

                DoctorStatus::Success
            }
        }
    }
}

struct InstallationScriptCheck;
#[async_trait]
impl DoctorCheck<DiagnosticsResponse> for InstallationScriptCheck {
    fn name(&self) -> String { "Installation script".to_string() }
    async fn check(&self, diagnostics: &DiagnosticsResponse) -> DoctorStatus {
        if diagnostics.installscript == "true" {
            DoctorStatus::Success
        } else {
            DoctorStatus::Error {
                reason: "Intall script not run".to_string(),
                info: vec![],
                fix: command_fix(vec!["~/.fig/tools/install_and_upgrade.sh"])
            }
        }

    }
}

struct ShellCompatibilityCheck;
#[async_trait]
impl DoctorCheck<DiagnosticsResponse> for ShellCompatibilityCheck {
    fn name(&self) -> String { "Compatible shell".to_string() }
    async fn check_basic(&self, _: &DiagnosticsResponse) -> Result<Option<String>> {
        let shell_regex = Regex::new(r"(bash|fish|zsh)").unwrap();
        let current_shell = get_shell();
        let valid_current_shell = match current_shell.as_ref() {
            Ok(shell) => {
                if shell_regex.is_match(&shell) {
                    Ok(None)
                } else {
                    Err(anyhow!("Current shell {} incompatible", shell))
                }
            }
            Err(_) => Ok(Some("Could not get current shell".to_string()))
        }?;

        let valid_default_shell = match dscl_read("UserShell").as_ref() {
            Ok(shell) => {
                if shell_regex.is_match(&shell) {
                    Ok(None)
                } else {
                    Err(anyhow!("Default shell {} incompatible", shell))
                }
            }
            Err(_) => Ok(Some("Could not get default shell".to_string()))
        }?;

        Ok(valid_current_shell.or(valid_default_shell))
    }
}

struct BundlePathCheck;
#[async_trait]
impl DoctorCheck<DiagnosticsResponse> for BundlePathCheck {
    fn name(&self) -> String { "Fig app installed in the right place".to_string() }
    async fn check(&self, diagnostics: &DiagnosticsResponse) -> DoctorStatus {
        let path = diagnostics.path_to_bundle.clone();
        if path.contains("/Applications/Fig.app") {
            DoctorStatus::Success
        } else if path.contains("/Build/Products/Debug/fig.app") {
            DoctorStatus::Warning(format!("Running debug build in {}", path.bold()))
        } else {
            DoctorStatus::Error {
                reason: format!("Fig app is installed in {}", path.bold()),
                info: vec![
                    "You need to install Fig in /Applications.".to_string(),
                    "To fix: uninstall, then reinstall Fig.".to_string(),
                    "Remember to drag Fig into the Applications folder.".to_string()
                ],
                fix: None
            }
        }
    }
}

struct AutocompleteEnabledCheck;
#[async_trait]
impl DoctorCheck<DiagnosticsResponse> for AutocompleteEnabledCheck {
    fn name(&self) -> String { "Autocomplete is enabled".to_string() }
    async fn check(&self, diagnostics: &DiagnosticsResponse) -> DoctorStatus {
        if diagnostics.autocomplete {
            DoctorStatus::Success
        } else {
            DoctorStatus::Error {
                reason: "Autocomplete disabled.".to_string(),
                info: vec![
                    format!("To fix run: {}", "fig settings autocomplete.disable false".magenta()),
                ],
                fix: None
            }
        }
    }
}

struct FigCLIPathCheck;
#[async_trait]
impl DoctorCheck<DiagnosticsResponse> for FigCLIPathCheck {
    fn name(&self) -> String { "Fig CLI path".to_string() }
    async fn check_basic(&self, _: &DiagnosticsResponse) -> Result<Option<String>> {
        let path = std::env::current_exe()
            .context("Could not get executable path.")?
            .to_string_lossy().to_string();
        let exe_path = fig_dir().unwrap().join("bin").join("fig")
            .to_string_lossy().to_string();
        if path != exe_path && path != "/usr/local/bin/.fig/bin/fig" && path != "/usr/local/bin/fig" {
            Ok(None)
        } else {
            anyhow::bail!("Fig CLI must be in {}", exe_path)
        }
    }
}

struct AccessibilityCheck;
#[async_trait]
impl DoctorCheck<DiagnosticsResponse> for AccessibilityCheck {
    fn name(&self) -> String { "Accessibility enabled".to_string() }
    async fn check(&self, diagnostics: &DiagnosticsResponse) -> DoctorStatus {
        if diagnostics.accessibility != "true" {
            DoctorStatus::Error {
                reason: "Accessibility is disabled".to_string(),
                info: vec![],
                fix: command_fix(vec!["fig", "debug", "prompt-accessibility"])
            }
        } else {
            DoctorStatus::Success
        }
    }
}

struct PseudoTerminalPathCheck;
#[async_trait]
impl DoctorCheck<DiagnosticsResponse> for PseudoTerminalPathCheck {
    fn name(&self) -> String { "PATH and PseudoTerminal PATH match".to_string() }
    async fn check(&self, diagnostics: &DiagnosticsResponse) -> DoctorStatus {
        let path = std::env::var("PATH").unwrap_or("".to_string());
        if diagnostics.psudoterminal_path.ne(&path) {
            DoctorStatus::Error {
                reason: "paths do not match".to_string(),
                info: vec![],
                fix: command_fix(vec!["fig", "app", "set-path"]),
            }
        } else {
            DoctorStatus::Success
        }
    }
}

struct DotfilesSymlinkedCheck;
#[async_trait]
impl DoctorCheck<DiagnosticsResponse> for DotfilesSymlinkedCheck {
    fn name(&self) -> String { "Dotfiles symlinked".to_string() }
    fn should_check(&self, diagnostics: &DiagnosticsResponse) -> bool {
        diagnostics.symlinked == "true"
    }
    async fn check_basic(&self, _: &DiagnosticsResponse) -> Result<Option<String>> {
        Ok(Some(
					"It looks like your dotfiles are symlinked. If you need to make modifications, make sure they're made in the right place.".to_string()
        ))
    }
}

struct SecureKeyboardCheck;
#[async_trait]
impl DoctorCheck<DiagnosticsResponse> for SecureKeyboardCheck {
    fn name(&self) -> String { "Secure keyboard input disabled".to_string() }
    async fn check(&self, diagnostics: &DiagnosticsResponse) -> DoctorStatus {
        if diagnostics.securekeyboard == "false" {
            return DoctorStatus::Success;
        }

        let mut info = vec![
            format!("Secure keyboard process is {}", diagnostics.securekeyboard_path)
        ];

        if is_installed("com.bitwarden.desktop") {
            let version = app_version("com.bitwarden.desktop");
            match version {
                Some(version) => {
                    if version <= Version::new(1, 27, 0) {
                        return DoctorStatus::Error {
                            reason: "Secure keyboard input is on".to_string(),
                            info: vec![
                                "Bitwarden may be enabling secure keyboard entry even when not focused.".to_string(),
                                "This was fixed in version 1.28.0. See https://github.com/bitwarden/desktop/issues/991 for details.".to_string(),
                                "To fix: upgrade Bitwarden to the latest version".to_string()

                            ],
                            fix: None
                        }
                    }
                }
                None => {
                    info.insert(0, "Could not get Bitwarden version".to_string());
                }
            }
        }

        DoctorStatus::Error {
            reason: "Secure keyboard input is on".to_string(),
            info: info,
            fix: None
        }
    }
}

struct ItermIntegrationCheck;
#[async_trait]
impl DoctorCheck for ItermIntegrationCheck {
    fn name(&self) -> String { "iTerm integration is enabled".to_string() }
    fn should_check(&self, _: &()) -> bool { is_installed("com.googlecode.iterm2") }
    async fn check_basic(&self, _: &()) -> Result<Option<String>> {
				// iTerm Integration
        let integration = verify_integration("com.googlecode.iterm2").await
            .context("Could not verify iTerm integration")?;
        if integration != "installed!" {
            let output = Command::new("defaults")
                .args(["read", "com.googlecode.iterm2", "EnableAPIServer"])
                .output();
            match output {
                Ok(output) => {
                    let api_enabled = String::from_utf8_lossy(&output.stdout);
                    if api_enabled.trim() == "0" {
                        anyhow::bail!("iTerm API server is not enabled.")
                    }
                }
                Err(_) => anyhow::bail!("Could not get iTerm API status")
            }

            let integration_path = home_dir()?
                .join("Library/Application Support/iTerm2/Scripts/AutoLaunch/fig-iterm-integration.scpt");
            if !integration_path.exists() {
                anyhow::bail!("fig-iterm-integration.scpt is missing.")
            }

            anyhow::bail!("Unknown error with iTerm integration");
        }

        let version = app_version("com.googlecode.iterm2")
            .ok_or(anyhow!("Could not get version"))?;
        if version < Version::new(3, 4, 0) {
            anyhow::bail!("iTerm version is incompatible with Fig. Please update iTerm to latest version");
        }
        Ok(None)
    }
}

struct ItermBashIntegrationCheck;
#[async_trait]
impl DoctorCheck for ItermBashIntegrationCheck {
    fn name(&self) -> String { "iTerm bash integration configured".to_string() }
    fn should_check(&self, _: &()) -> bool {
        match home_dir() {
            Ok(home) => home.join(".iterm2_shell_integration.bash").exists(),
            Err(_) => false
        }
    }
    async fn check_basic(&self, _: &()) -> Result<Option<String>> {
        let integration_file = home_dir()?.join(".iterm2_shell_integration.bash");
        let integration = read_to_string(integration_file)
            .context("Could not read .iterm2_shell_integration.bash")?;

        match Regex::new(r"V(\d*\.\d*\.\d*)").unwrap().captures(&integration) {
            Some(captures) => {
                let version = captures.get(1).unwrap().as_str();
                if Version::new(0, 4, 0) > Version::parse(version).unwrap() {
							      anyhow::bail!(
                        "iTerm Bash Integration is out of date. Please update in iTerm's menu by selecting \"Install Shell Integration\"."
							      )
                }
                Ok(None)
            }
            None => {
						    Ok(Some(
                    "iTerm's Bash Integration is installed, but we could not check the version in ~/.iterm2_shell_integration.bash. Integration may be out of date. You can try updating in iTerm's menu by selecting \"Install Shell Integration\"".to_string()
						    ))
            }
        }
    }
}

struct HyperIntegrationCheck;
#[async_trait]
impl DoctorCheck for HyperIntegrationCheck {
    fn name(&self) -> String { "Hyper integration is enabled".to_string() }
    fn should_check(&self, _: &()) -> bool { is_installed("co.zeit.hyper") }
    async fn check_basic(&self, _: &()) -> Result<Option<String>> {
        let integration = verify_integration("co.zeit.hyper").await
            .context("Could not verify Hyper integration")?;
        if integration != "installed!" {
            // Check ~/.hyper_plugins/local/fig-hyper-integration/index.js exists
            let integration_path = home_dir()?.join(".hyper_plugins/local/fig-hyper-integration/index.js");

            if !integration_path.exists() {
                anyhow::bail!("fig-hyper-integration plugin is missing.");
            }

            let config = read_to_string(home_dir()?.join(".hyper.js"))
                .context("Could not read ~/.hyper.js")?;

            if !config.contains("fig-hyper-integration") {
                anyhow::bail!("fig-hyper-integration plugin needs to be added to localPlugins!")
            }
            anyhow::bail!("Unknown error with integration!")
        }

        Ok(None)
    }
}

struct SystemVersionCheck;
#[async_trait]
impl DoctorCheck for SystemVersionCheck {
    fn name(&self) -> String { "OS is supported".to_string() }
    async fn check_basic(&self, _: &()) -> Result<Option<String>> {
        let os_version = get_os_version()
            .context("Could not get OS Version")?;
        if !os_version.is_supported() {
            anyhow::bail!("{} is not supported", os_version.to_string())
        } else {
            Ok(None)
        }
    }
}


struct VSCodeIntegrationCheck;
#[async_trait]
impl DoctorCheck for VSCodeIntegrationCheck {
    fn name(&self) -> String { "VSCode integration is enabled".to_string() }
    fn should_check(&self, _: &()) -> bool { is_installed("com.microsoft.VSCode") }
    async fn check_basic(&self, _: &()) -> Result<Option<String>> {
        let integration = verify_integration("com.microsoft.VSCode").await
            .context("Could not verify VSCode integration")?;
        if integration != "installed!" {
            // Check if withfig.fig exists
            let extensions = home_dir()?.join(".vscode").join("extensions");

            let glob_set = glob(&[extensions.join("withfig.fig-").to_string_lossy().to_string()]).unwrap();
            let fig_extensions = glob_dir(&glob_set, extensions)
                .context("Could not read extensions")?;
            if fig_extensions.len() == 0 {
                anyhow::bail!("VSCode extension is missing!")
            }
            anyhow::bail!("Unknown error with integration!")
        }

        Ok(None)
    }
}

struct LoginStatusCheck;
#[async_trait]
impl DoctorCheck for LoginStatusCheck {
    fn name(&self) -> String { "Logged into Fig".to_string() }
    async fn check_basic(&self, _: &()) -> Result<Option<String>> {
        if let Ok(creds) = Credentials::load_credentials() {
            if creds.get_access_token().is_some() &&
                creds.get_id_token().is_some() &&
                creds.get_refresh_token().is_some() {
                return Ok(None);
            }
        }
        anyhow::bail!("Not logged in. Run `dotfiles login` to login.");
    }
}

async fn run_checks_with_context<T, Fut>(
    header: String,
    checks: Vec<&dyn DoctorCheck<T>>,
    get_context: impl Fn() -> Fut
) -> Result<()>
where
    T: Sync + Send,
    Fut: Future<Output = Result<T>>,
{
    println!("{}", header.dark_grey());
    let mut context = get_context().await?;
    for check in checks {
        let name = check.name();
        if !check.should_check(&context) {
            continue;
        }
        let result = check.check(&context).await;
        print_status_result(&name, &result);
        if let DoctorStatus::Error { reason, info: _, fix } = result {
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
                        DoctorStatus::Error { reason: _, info: _, fix: _ } => {},
                        _ => { continue; }
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

    async fn run_all_checks() -> Result<()> {
        run_checks(
            "Let's make sure Fig is running...".to_string(),
            vec![
                &FigBinCheck {},
                &PathCheck {},
                &AppRunningCheck {},
                &FigSocketCheck {},
                &FigtermSocketCheck {}
            ],
        ).await?;
        run_checks(
            "Let's check your dotfiles...".to_string(),
            vec![
                // TODO
                &DotfileCheck { shell: Shell::Bash, path: Shell::Bash.get_config_path()? },
                &DotfileCheck { shell: Shell::Zsh, path: Shell::Zsh.get_config_path()? }
            ],
        ).await?;
        run_checks(
            "Let's check if your system is compatible...".to_string(),
            vec![
                &SystemVersionCheck {}
            ],
        ).await?;
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
                &DotfilesSymlinkedCheck {}
            ],
            get_diagnostics
        ).await?;
        run_checks(
            "Let's check your integrations...".to_string(),
            vec![
                &ItermIntegrationCheck {},
                &ItermBashIntegrationCheck {},
                &HyperIntegrationCheck {},
                &VSCodeIntegrationCheck {},
            ],
        ).await?;
        run_checks(
            "Let's check if you're logged in...".to_string(),
            vec![
                &LoginStatusCheck {},
            ],
        ).await?;

        Ok(())
    }

    if run_all_checks().await.is_err() {
        println!();
        println!("❌ Doctor found errors. Please fix them and try again.");
        println!();
        println!("If you are not sure how to fix it, please open an issue with {} to let us know!", "fig issue".magenta());
        println!("Or, email us at {}!", "hello@fig.io".underlined().dark_cyan());
        println!()
    } else {
        println!();
        println!("✅ Everything looks good!");
        println!();
        println!("Fig still not working? Run {} to let us know!", "fig issue".magenta());
        println!("Or, email us at {}!", "hello@fig.io".underlined().dark_cyan());
        println!()
    }

    Ok(())
}
