use std::borrow::Cow;
use std::ffi::OsStr;
use std::fmt::Display;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{
    Context,
    Result,
};
use crossterm::style::Stylize;
use fig_ipc::command::send_recv_command_to_socket;
use fig_proto::local::command_response::Response;
use fig_proto::local::{
    command,
    DiagnosticsCommand,
    DiagnosticsResponse,
    IntegrationAction,
    TerminalIntegrationCommand,
};
use regex::Regex;
use serde::{
    Deserialize,
    Serialize,
};

use crate::cli::OutputFormat;
use crate::util::{
    glob,
    glob_dir,
    is_app_running,
    OSVersion,
};

pub trait Diagnostic {
    fn user_readable(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }
}

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

fn get_local_specs() -> Result<Vec<PathBuf>> {
    let specs_location = fig_directories::home_dir()
        .context("Could not get home dir")?
        .join(".fig")
        .join("autocomplete");
    let glob_pattern = specs_location.join("*.js");
    let patterns = [glob_pattern.to_str().unwrap()];
    let glob = glob(&patterns)?;

    glob_dir(&glob, specs_location)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HardwareInfo {
    model_name: Option<String>,
    model_identifier: Option<String>,
    chip: Option<String>,
    total_cores: Option<String>,
    memory: Option<String>,
}

impl HardwareInfo {
    fn new() -> Result<HardwareInfo> {
        cfg_if! {
            if #[cfg(target_os = "macos")] {
                use crate::util::match_regex;

                let result = Command::new("system_profiler")
                    .arg("SPHardwareDataType")
                    .output()
                    .with_context(|| "Could not read hardware")?;

                let text: String = String::from_utf8_lossy(&result.stdout).trim().into();

                Ok(HardwareInfo {
                    model_name: match_regex(r"Model Name: (.+)", &text),
                    model_identifier: match_regex(r"Model Identifier: (.+)", &text),
                    chip: match_regex(r"Chip: (.+)", &text),
                    total_cores: match_regex(r"Total Number of Cores: (.+)", &text),
                    memory: match_regex(r"Memory: (.+)", &text),
                })
            } else {
                use sysinfo::{System, SystemExt, CpuExt};

                let mut sys = System::new();
                sys.refresh_cpu();
                sys.refresh_memory();

                let mut hardware_info = HardwareInfo {
                    model_name: None,
                    model_identifier: None,
                    chip: None,
                    total_cores: Some(
                        sys.physical_core_count()
                            .map_or_else(|| "Unknown".into(), |cores| format!("{cores}")),
                    ),
                    memory: Some(format!("{} KB", sys.total_memory())),
                };

                if let Some(processor) = sys.cpus().first() {
                    hardware_info.model_name = Some(processor.name().into());
                    hardware_info.model_identifier = Some(processor.vendor_id().into());
                    hardware_info.chip = Some(processor.brand().into());
                }

                Ok(hardware_info)
            }
        }
    }
}

impl Diagnostic for HardwareInfo {
    fn user_readable(&self) -> Result<Vec<String>> {
        Ok(vec![
            format!("Model Name: {}", self.model_name.as_deref().unwrap_or_default()),
            format!(
                "Model Identifier: {}",
                self.model_identifier.as_deref().unwrap_or_default()
            ),
            format!("Chip: {}", self.chip.as_deref().unwrap_or_default()),
            format!("Cores: {}", self.total_cores.as_deref().unwrap_or_default()),
            format!("Memory: {}", self.memory.as_deref().unwrap_or_default()),
        ])
    }
}

impl Diagnostic for OSVersion {
    fn user_readable(&self) -> Result<Vec<String>> {
        Ok(vec![format!("{}", self)])
    }
}

fn installed_via_brew() -> Result<bool> {
    let result = Command::new("brew")
        .arg("list")
        .arg("--cask")
        .output()
        .with_context(|| "Could not get brew casks")?;
    let text = String::from_utf8_lossy(&result.stdout);

    Ok(Regex::new(r"(?m:^fig$)").unwrap().is_match(text.trim()))
}

pub async fn get_diagnostics() -> Result<DiagnosticsResponse> {
    let response = send_recv_command_to_socket(command::Command::Diagnostics(DiagnosticsCommand {}))
        .await?
        .context("Recieved EOF while reading diagnostics")?;

    match response.response {
        Some(Response::Diagnostics(diagnostics)) => Ok(diagnostics),
        _ => anyhow::bail!("Invalid response"),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Version {
    distribution: String,
    beta: bool,
    debug_mode: bool,
    dev_mode: bool,
    layout: String,
    is_running_on_read_only_volume: bool,
}

impl Diagnostic for Version {
    fn user_readable(&self) -> Result<Vec<String>> {
        let mut version: Vec<Cow<str>> = vec![self.distribution.as_str().into()];

        if self.beta {
            version.push("[Beta]".into())
        }
        if self.debug_mode {
            version.push("[Debug]".into())
        }
        if self.dev_mode {
            version.push("[Dev]".into())
        }

        if !self.layout.is_empty() {
            version.push(format!("[{}]", self.layout).into());
        }

        if self.is_running_on_read_only_volume {
            version.push("TRANSLOCATED!".into());
        }

        Ok(vec![format!("Fig version: {}", version.join(" "))])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EnvVarDiagnostic {
    envs: Vec<(String, String)>,
}

impl EnvVarDiagnostic {
    fn new() -> EnvVarDiagnostic {
        let envs = std::env::vars()
            .into_iter()
            .filter(|(key, _)| key.starts_with("FIG_") || key == "TERM_SESSION_ID" || key == "PATH" || key == "TERM")
            .collect();

        EnvVarDiagnostic { envs }
    }
}

impl Diagnostic for EnvVarDiagnostic {
    fn user_readable(&self) -> Result<Vec<String>> {
        let mut lines = vec![];

        for (key, value) in &self.envs {
            lines.push(format!("{}={}", key, value));
        }

        Ok(lines)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CurrentEnvironment {
    user_shell: String,
    current_dir: String,
    cli_installed: bool,
    executable_location: String,
    installed_via_brew: Option<bool>,
    current_window_id: Option<String>,
    current_process: Option<String>,
}

impl CurrentEnvironment {
    fn new() -> CurrentEnvironment {
        let user_shell = fig_settings::state::get_value("userShell")
            .ok()
            .flatten()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| "Unknown UserShell".into());

        let current_dir = std::env::current_dir()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "Could not get working directory".into());

        let executable_location = std::env::current_exe()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "<unknown>".into());

        CurrentEnvironment {
            user_shell,
            current_dir,
            cli_installed: true,
            executable_location,
            installed_via_brew: installed_via_brew().ok(),
            current_window_id: None,
            current_process: None,
        }
    }

    fn current_window_id(&mut self, window_id: impl Into<String>) {
        self.current_window_id = Some(window_id.into());
    }

    fn current_process(&mut self, current_process: impl Into<String>) {
        self.current_process = Some(current_process.into());
    }
}

impl Diagnostic for CurrentEnvironment {
    fn user_readable(&self) -> Result<Vec<String>> {
        let mut lines = vec![
            format!("User Shell: {}", self.user_shell),
            format!("Current Directory: {}", self.current_dir),
            format!("CLI Installed: {}", self.cli_installed),
            format!("Executable Location: {}", self.executable_location),
            format!(
                "Current Window ID: {}",
                self.current_window_id.as_deref().unwrap_or("<unknown>")
            ),
            format!(
                "Active Process: {}",
                self.current_process.as_deref().unwrap_or("<unknown>")
            ),
        ];

        if let Some(true) = self.installed_via_brew {
            lines.push("Installed via Brew: true".into());
        }

        Ok(lines)
    }
}

pub async fn verify_integration(integration: impl Into<String>) -> Result<String> {
    let response = send_recv_command_to_socket(command::Command::TerminalIntegration(TerminalIntegrationCommand {
        identifier: integration.into(),
        action: IntegrationAction::VerifyInstall as i32,
    }))
    .await?
    .context("Recieved EOF while getting terminal integration")?;

    let message = match response.response {
        Some(Response::Success(success)) => success.message,
        Some(Response::Error(error)) => error.message,
        _ => anyhow::bail!("Invalid response"),
    };

    message.context("No message found")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Integrations {
    Ssh,
    Tmux,
    ITerm,
    Hyper,
    VsCode,
    Docker,
}

impl Display for Integrations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Integrations::Ssh => f.write_str("SSH"),
            Integrations::Tmux => f.write_str("TMUX"),
            Integrations::ITerm => f.write_str("iTerm"),
            Integrations::Hyper => f.write_str("Hyper"),
            Integrations::VsCode => f.write_str("Visual Studio Code"),
            Integrations::Docker => f.write_str("Docker"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IntegrationStatus {
    status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IntegrationDiagnostics {
    integrations: Vec<(Integrations, IntegrationStatus)>,
}

impl IntegrationDiagnostics {
    async fn new() -> IntegrationDiagnostics {
        let mut integrations = vec![
            (Integrations::Ssh, IntegrationStatus { status: "false".into() }),
            (Integrations::Tmux, IntegrationStatus { status: "false".into() }),
        ];

        let integration_result = verify_integration("com.googlecode.iterm2")
            .await
            .unwrap_or_else(|e| format!("Error {}", e));

        integrations.push((Integrations::ITerm, IntegrationStatus {
            status: integration_result,
        }));

        let integration_result = verify_integration("co.zeit.hyper")
            .await
            .unwrap_or_else(|e| format!("Error {}", e));

        integrations.push((Integrations::Hyper, IntegrationStatus {
            status: integration_result,
        }));

        let integration_result = verify_integration("com.microsoft.VSCode")
            .await
            .unwrap_or_else(|e| format!("Error {}", e));

        integrations.push((Integrations::VsCode, IntegrationStatus {
            status: integration_result,
        }));

        IntegrationDiagnostics { integrations }
    }

    fn docker(&mut self, status: impl Into<String>) {
        self.integrations
            .push((Integrations::Docker, IntegrationStatus { status: status.into() }));
    }
}

impl Diagnostic for IntegrationDiagnostics {
    fn user_readable(&self) -> Result<Vec<String>> {
        let mut lines = vec![];

        for (integration, status) in &self.integrations {
            lines.push(format!("{}: {}", integration, status.status));
        }

        Ok(lines)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FigDetails {
    path_to_bundle: String,
    autocomplete: bool,
    settings_json: bool,
    accessibility: String,
    num_specs: usize,
    symlinked: String,
    onlytab: String,
    keypath: String,
    installscript: String,
    psudoterminal_path: String,
    securekeyboard: String,
    securekeyboard_path: String,
    insertion_lock: Option<String>,
}

impl FigDetails {
    fn new(diagnostics: &DiagnosticsResponse) -> FigDetails {
        FigDetails {
            path_to_bundle: diagnostics.path_to_bundle.clone(),
            autocomplete: diagnostics.autocomplete,
            settings_json: fig_settings::settings::local_settings().is_ok(),
            accessibility: diagnostics.accessibility.clone(),
            num_specs: get_local_specs().map_or(0, |v| v.len()),
            symlinked: diagnostics.symlinked.clone(),
            onlytab: diagnostics.onlytab.clone(),
            keypath: diagnostics.keypath.clone(),
            installscript: diagnostics.installscript.clone(),
            psudoterminal_path: diagnostics.psudoterminal_path.clone(),
            securekeyboard: diagnostics.securekeyboard.clone(),
            securekeyboard_path: diagnostics.securekeyboard_path.clone(),
            insertion_lock: None,
        }
    }
}

impl Diagnostic for FigDetails {
    fn user_readable(&self) -> Result<Vec<String>> {
        let mut lines = vec![];

        lines.push(format!("Bundle path: {}", self.path_to_bundle));
        lines.push(format!("Autocomplete: {}", self.autocomplete));
        lines.push(format!("Settings.json: {}", self.settings_json));
        lines.push(format!("Accessibility: {}", self.accessibility));
        lines.push(format!("Number of specs: {}", self.num_specs));
        lines.push(format!("Symlinked dotfiles: {}", self.symlinked));
        lines.push(format!("Only insert on tab: {}", self.onlytab));
        lines.push(format!("Keybindings path: {}", self.keypath));
        lines.push(format!("Installation Script: {}", self.installscript));
        lines.push(format!("PseudoTerminal Path: {}", self.psudoterminal_path));
        lines.push(format!("SecureKeyboardInput: {}", self.securekeyboard));
        lines.push(format!("SecureKeyboardProcess: {}", self.securekeyboard_path));

        Ok(lines)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DotfilesDiagnostics {
    profile: Option<String>,
    bashrc: Option<String>,
    bash_profile: Option<String>,
    zshrc: Option<String>,
    zprofile: Option<String>,
}

impl DotfilesDiagnostics {
    fn new() -> Result<DotfilesDiagnostics> {
        let home_dir = fig_directories::home_dir().context("Could not get base dir")?;

        let profile = std::fs::read_to_string(home_dir.join(".profile")).ok();
        let bashrc = std::fs::read_to_string(home_dir.join(".bashrc")).ok();
        let bash_profile = std::fs::read_to_string(home_dir.join(".bash_profile")).ok();
        let zshrc = std::fs::read_to_string(home_dir.join(".zshrc")).ok();
        let zprofile = std::fs::read_to_string(home_dir.join(".zprofile")).ok();

        Ok(DotfilesDiagnostics {
            profile,
            bashrc,
            bash_profile,
            zshrc,
            zprofile,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostics {
    timestamp: u64,
    fig_running: bool,
    version: Option<Version>,
    hardware: HardwareInfo,
    os: OSVersion,
    user_env: CurrentEnvironment,
    env_var: EnvVarDiagnostic,
    fig_details: Option<FigDetails>,
    integrations: Option<IntegrationDiagnostics>,
    dotfiles: DotfilesDiagnostics,
}

impl Diagnostics {
    pub async fn new() -> Result<Diagnostics> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        match get_diagnostics().await {
            Ok(diagnostics) => {
                let mut integrations = IntegrationDiagnostics::new().await;
                integrations.docker(&diagnostics.docker);

                let mut current_env = CurrentEnvironment::new();
                current_env.current_window_id(&diagnostics.current_window_identifier);
                current_env.current_process(&diagnostics.current_process);

                let fig_details = FigDetails::new(&diagnostics);

                Ok(Diagnostics {
                    timestamp,
                    fig_running: true,
                    version: Some(Version {
                        distribution: diagnostics.distribution,
                        beta: diagnostics.beta,
                        debug_mode: diagnostics.debug_autocomplete,
                        dev_mode: diagnostics.developer_mode_enabled,
                        layout: diagnostics.current_layout_name,
                        is_running_on_read_only_volume: diagnostics.is_running_on_read_only_volume,
                    }),
                    hardware: HardwareInfo::new()?,
                    os: OSVersion::new()?,
                    user_env: current_env,
                    env_var: EnvVarDiagnostic::new(),
                    fig_details: Some(fig_details),
                    integrations: Some(integrations),
                    dotfiles: DotfilesDiagnostics::new()?,
                })
            },
            _ => Ok(Diagnostics {
                timestamp,
                fig_running: false,
                version: None,
                hardware: HardwareInfo::new()?,
                os: OSVersion::new()?,
                user_env: CurrentEnvironment::new(),
                env_var: EnvVarDiagnostic::new(),
                fig_details: None,
                integrations: None,
                dotfiles: DotfilesDiagnostics::new()?,
            }),
        }
    }
}

impl Diagnostic for Diagnostics {
    fn user_readable(&self) -> Result<Vec<String>> {
        let print_indent = |lines: &[String], indent: &str, level: usize| {
            let mut new_lines = vec![];
            for line in lines {
                new_lines.push(format!("{}- {}", indent.repeat(level), line));
            }
            new_lines
        };

        let mut lines = vec!["# Fig Diagnostics".into()];

        if !self.fig_running {
            lines.push("## NOTE: Fig is not running, run `fig launch` to get the full diagnostics".into());
        }

        lines.push("## Fig details:".into());
        if let Some(version) = &self.version {
            lines.extend(print_indent(&version.user_readable()?, "  ", 1));
        }
        if let Some(details) = &self.fig_details {
            lines.extend(print_indent(&details.user_readable()?, "  ", 1));
        }
        lines.push("## Hardware Info:".into());
        lines.extend(print_indent(&self.hardware.user_readable()?, "  ", 1));
        lines.push("## OS Info:".into());
        lines.extend(print_indent(&self.os.user_readable()?, "  ", 1));
        lines.push("## Environment:".into());
        lines.extend(print_indent(&self.user_env.user_readable()?, "  ", 1));
        lines.push("  - Environment Variables:".into());
        lines.extend(print_indent(&self.env_var.user_readable()?, "  ", 2));
        lines.push("## Integrations:".into());
        if let Some(integrations) = &self.integrations {
            lines.extend(print_indent(&integrations.user_readable()?, "  ", 1));
        }
        Ok(lines)
    }
}

pub async fn diagnostics_cli(format: OutputFormat, force: bool) -> Result<()> {
    if !force && !is_app_running() {
        println!(
            "\n→ Fig is not running.\n  Please launch Fig with {} or run {} to get limited diagnostics.",
            "fig launch".magenta(),
            "fig diagnostic --force".magenta()
        );
        return Ok(());
    }

    let diagnostics = Diagnostics::new().await?;

    match format {
        OutputFormat::Plain => println!("{}", diagnostics.user_readable()?.join("\n")),
        OutputFormat::Json => println!("{}", serde_json::to_string(&diagnostics)?),
        OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&diagnostics)?),
    }

    Ok(())
}
