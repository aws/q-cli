use std::fmt::Display;
use std::process::Command;

use atty;
use cfg_if::cfg_if;
use clap::Args;
use crossterm::terminal::{
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
use fig_ipc::local::send_recv_command_to_socket;
use fig_proto::local::command_response::Response;
use fig_proto::local::{
    command,
    IntegrationAction,
    TerminalIntegrationCommand,
};
#[cfg(unix)]
use fig_proto::local::{
    DiagnosticsCommand,
    DiagnosticsResponse,
};
use fig_telemetry::InstallMethod;
use fig_util::system_info::OSVersion;
use fig_util::{
    directories,
    Terminal,
};
use serde::{
    Deserialize,
    Serialize,
};
use spinners::{
    Spinner,
    Spinners,
};

use crate::cli::OutputFormat;
#[derive(Debug, Args, PartialEq, Eq)]
pub struct DiagnosticArgs {
    /// The format of the output
    #[arg(long, short, value_enum, default_value_t)]
    format: OutputFormat,
    /// Force limited diagnostic output
    #[arg(long)]
    force: bool,
}

impl DiagnosticArgs {
    pub async fn execute(&self) -> Result<()> {
        #[cfg(target_os = "macos")]
        if !self.force && !fig_util::is_fig_desktop_running() {
            use owo_colors::OwoColorize;

            println!(
                "\n→ Fig is not running.\n  Please launch Fig with {} or run {} to get limited diagnostics.",
                "fig launch".magenta(),
                "fig diagnostic --force".magenta()
            );
            return Ok(());
        }

        let spinner = if atty::is(atty::Stream::Stdout) {
            Some(Spinner::new(Spinners::Dots, "Generating...".into()))
        } else {
            None
        };

        if spinner.is_some() {
            execute!(std::io::stdout(), cursor::Hide)?;

            ctrlc::set_handler(move || {
                execute!(std::io::stdout(), cursor::Show).ok();
                std::process::exit(1);
            })?;
        }

        let diagnostics = Diagnostics::new().await?;

        if let Some(mut sp) = spinner {
            sp.stop();
            execute!(std::io::stdout(), Clear(ClearType::CurrentLine), cursor::Show)?;
            println!();
        }

        match self.format {
            OutputFormat::Plain => println!("{}", diagnostics.user_readable()?.join("\n")),
            OutputFormat::Json => println!("{}", serde_json::to_string(&diagnostics)?),
            OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&diagnostics)?),
        }

        Ok(())
    }
}

pub trait Diagnostic {
    fn user_readable(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NewFigDetails {
    cli_version: String,
    desktop_version: Option<String>,
    figterm_version: Option<String>,
}

impl NewFigDetails {
    pub fn new() -> NewFigDetails {
        let desktop_version = Command::new("fig_desktop")
            .arg("--version")
            .output()
            .ok()
            .map(|output| {
                String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .replace("fig_desktop ", "")
            });

        let figterm_version = Command::new("figterm")
            .arg("--version")
            .output()
            .ok()
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().replace("figterm ", ""));

        NewFigDetails {
            cli_version: env!("CARGO_PKG_VERSION").into(),
            desktop_version,
            figterm_version,
        }
    }
}

impl Diagnostic for NewFigDetails {
    fn user_readable(&self) -> Result<Vec<String>> {
        let mut details = vec![format!("cli-version: {}", self.cli_version)];

        if let Some(ref desktop_version) = self.desktop_version {
            details.push(format!("desktop-version: {desktop_version}"));
        }

        if let Some(ref figterm_version) = self.figterm_version {
            details.push(format!("figterm-version: {figterm_version}"));
        }

        Ok(details)
    }
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
                    memory: Some(format!("{:0.2} GB", sys.total_memory() as f32 / 2.0_f32.powi(30))),
                };

                if let Some(processor) = sys.cpus().first() {
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
            #[cfg(target_os = "macos")]
            format!("model: {}", self.model_name.as_deref().unwrap_or_default()),
            #[cfg(target_os = "macos")]
            format!("model-id: {}", self.model_identifier.as_deref().unwrap_or_default()),
            format!("chip-id: {}", self.chip.as_deref().unwrap_or_default()),
            format!("cores: {}", self.total_cores.as_deref().unwrap_or_default()),
            format!("mem: {}", self.memory.as_deref().unwrap_or_default()),
        ])
    }
}

impl Diagnostic for OSVersion {
    fn user_readable(&self) -> Result<Vec<String>> {
        match self {
            OSVersion::Linux {
                kernel_version,
                os_release,
            } => {
                let mut v = vec![format!("kernel: {kernel_version}")];

                if let Some(os_release) = os_release {
                    if let Some(name) = &os_release.name {
                        v.push(format!("distro: {name}"));
                    }

                    if let Some(version) = &os_release.version {
                        v.push(format!("distro-version: {version}"));
                    } else if let Some(version) = &os_release.version_id {
                        v.push(format!("distro-version: {version}"));
                    }

                    if let Some(variant) = &os_release.variant {
                        v.push(format!("distro-variant: {variant}"));
                    } else if let Some(variant) = &os_release.variant_id {
                        v.push(format!("distro-variant: {variant}"));
                    }

                    if let Some(build) = &os_release.build_id {
                        v.push(format!("distro-build: {build}"));
                    }
                }

                Ok(v)
            },
            other => Ok(vec![format!("{other}")]),
        }
    }
}

#[cfg(unix)]
pub async fn get_diagnostics() -> Result<DiagnosticsResponse> {
    let response = send_recv_command_to_socket(command::Command::Diagnostics(DiagnosticsCommand {}))
        .await?
        .context("Received EOF while reading diagnostics")?;

    match response.response {
        Some(Response::Diagnostics(diagnostics)) => Ok(diagnostics),
        _ => eyre::bail!("Invalid response"),
    }
}

#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Version {
    distribution: String,
    beta: bool,
    debug_mode: bool,
    dev_mode: bool,
    layout: String,
    is_running_on_read_only_volume: bool,
}

#[cfg(target_os = "macos")]
impl Diagnostic for Version {
    fn user_readable(&self) -> Result<Vec<String>> {
        let mut version: Vec<std::borrow::Cow<str>> = vec![self.distribution.as_str().into()];

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

        Ok(vec![format!("desktop-version: {}", version.join(" "))])
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
            .filter(|(key, _)| {
                key.starts_with("FIG_")
                    || key == "SHELL"
                    || key == "DISPLAY"
                    || key == "PATH"
                    || key == "FIGTERM_SESSION_ID"
                    || key == "TERM"
                    || key == "XDG_CURRENT_DESKTOP"
                    || key == "XDG_SESSION_DESKTOP"
                    || key == "XDG_SESSION_TYPE"
                    || key == "GLFW_IM_MODULE"
                    || key == "GTK_IM_MODULE"
                    || key == "QT_IM_MODULE"
                    || key == "XMODIFIERS"
            })
            .collect();

        EnvVarDiagnostic { envs }
    }
}

impl Diagnostic for EnvVarDiagnostic {
    fn user_readable(&self) -> Result<Vec<String>> {
        let mut lines = vec![];

        for (key, value) in &self.envs {
            lines.push(format!("{}: {}", key, value));
        }

        Ok(lines)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CurrentEnvironment {
    shell_exe: Option<String>,
    figterm_exe: Option<String>,
    terminal_exe: Option<String>,
    current_dir: String,
    executable_location: String,
    terminal: Option<Terminal>,
    install_method: InstallMethod,
}

impl CurrentEnvironment {
    fn new() -> CurrentEnvironment {
        use fig_util::process_info::{
            Pid,
            PidExt,
        };

        let self_pid = Pid::current();

        let shell_pid = self_pid.parent();
        let shell_exe = shell_pid.and_then(|pid| pid.exe()).map(|p| p.display().to_string());

        let current_dir = std::env::current_dir()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "Could not get working directory".into());

        let executable_location = std::env::current_exe()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "<unknown>".into());

        let terminal = fig_util::terminal::Terminal::parent_terminal();

        let install_method = fig_telemetry::get_install_method();

        CurrentEnvironment {
            shell_exe,
            figterm_exe: None,
            terminal_exe: None,
            current_dir,
            executable_location,
            terminal,
            install_method,
        }
    }
}

impl Diagnostic for CurrentEnvironment {
    fn user_readable(&self) -> Result<Vec<String>> {
        Ok(vec![
            format!("shell: {}", self.shell_exe.as_deref().unwrap_or("<unknown>")),
            format!(
                "terminal: {}",
                self.terminal
                    .as_ref()
                    .map(|term| term.internal_id())
                    .as_deref()
                    .unwrap_or("<unknown>")
            ),
            format!("cwd: {}", self.current_dir),
            format!("exe-path: {}", self.executable_location),
            format!("install-method: {}", self.install_method),
        ])
    }
}

pub async fn verify_integration(integration: impl Into<String>) -> Result<String> {
    let response = send_recv_command_to_socket(command::Command::TerminalIntegration(TerminalIntegrationCommand {
        identifier: integration.into(),
        action: IntegrationAction::VerifyInstall as i32,
    }))
    .await?
    .context("Received EOF while getting terminal integration")?;

    let message = match response.response {
        Some(Response::Success(success)) => success.message,
        Some(Response::Error(error)) => error.message,
        _ => eyre::bail!("Invalid response"),
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

#[allow(dead_code)]
impl IntegrationDiagnostics {
    #[cfg(target_os = "macos")]
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

    #[cfg(target_os = "macos")]
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
struct DotfilesDiagnostics {
    profile: Option<String>,
    bashrc: Option<String>,
    bash_profile: Option<String>,
    zshrc: Option<String>,
    zprofile: Option<String>,
}

impl DotfilesDiagnostics {
    fn new() -> Result<DotfilesDiagnostics> {
        let home_dir = directories::home_dir().context("Could not get base dir")?;

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

    #[cfg(target_os = "macos")]
    version: Option<Version>,

    new_fig_details: NewFigDetails,

    fig_running: bool,
    hardware: HardwareInfo,
    pub os: Option<OSVersion>,
    user_env: CurrentEnvironment,
    env_var: EnvVarDiagnostic,
    integrations: Option<IntegrationDiagnostics>,
    dotfiles: DotfilesDiagnostics,
}

impl Diagnostics {
    pub async fn new() -> Result<Diagnostics> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        cfg_if::cfg_if! {
            if #[cfg(target_os = "macos")] {
                match get_diagnostics().await {
                    Ok(diagnostics) => {
                        // TODO(sean) add back integration information once we have a better
                        // understanding of IME/terminal integrations.
                        // let mut integrations = IntegrationDiagnostics::new().await;
                        // integrations.docker(&diagnostics.docker);

                        let current_env = CurrentEnvironment::new();

                        Ok(Diagnostics {
                            timestamp,
                            fig_running: true,
                            new_fig_details: NewFigDetails::new(),
                            version: Some(Version {
                                distribution: diagnostics.distribution,
                                beta: diagnostics.beta,
                                debug_mode: diagnostics.debug_autocomplete,
                                dev_mode: diagnostics.developer_mode_enabled,
                                layout: diagnostics.current_layout_name,
                                is_running_on_read_only_volume: diagnostics.is_running_on_read_only_volume,
                            }),
                            hardware: HardwareInfo::new()?,
                            os: fig_util::system_info::os_version().cloned(),
                            user_env: current_env,
                            env_var: EnvVarDiagnostic::new(),
                            integrations: None,
                            dotfiles: DotfilesDiagnostics::new()?,
                        })
                    },
                    _ => Ok(Diagnostics {
                        timestamp,
                        fig_running: false,
                        new_fig_details: NewFigDetails::new(),
                        version: None,
                        hardware: HardwareInfo::new()?,
                        os: fig_util::system_info::os_version().cloned(),
                        user_env: CurrentEnvironment::new(),
                        env_var: EnvVarDiagnostic::new(),
                        integrations: None,
                        dotfiles: DotfilesDiagnostics::new()?,
                    }),
                }
            } else {
                Ok(Diagnostics {
                    timestamp,
                    fig_running: true,
                    new_fig_details: NewFigDetails::new(),
                    hardware: HardwareInfo::new()?,
                    os: fig_util::system_info::os_version().cloned(),
                    user_env: CurrentEnvironment::new(),
                    env_var: EnvVarDiagnostic::new(),
                    integrations: None,
                    dotfiles: DotfilesDiagnostics::new()?,
                })
            }
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

        let mut lines = vec![];

        if !self.fig_running {
            lines.push("## NOTE: Fig is not running, run `fig launch` to get the full diagnostics".into());
        }

        lines.push("fig-details:".into());
        cfg_if::cfg_if! {
            if #[cfg(target_os = "macos")] {
                if let Some(version) = &self.version {
                    lines.extend(print_indent(&version.user_readable()?, "  ", 1));
                }
            } else {
                lines.extend(print_indent(&self.new_fig_details.user_readable()?, "  ", 1));
            }
        }
        lines.push("hardware-info:".into());
        lines.extend(print_indent(&self.hardware.user_readable()?, "  ", 1));
        lines.push("os-info:".into());
        match self.os {
            Some(ref os) => lines.extend(print_indent(&os.user_readable()?, "  ", 1)),
            None => lines.push(format!("  - os: {}", std::env::consts::OS)),
        }
        lines.push("environment:".into());
        lines.extend(print_indent(&self.user_env.user_readable()?, "  ", 1));
        lines.push("  - env-vars:".into());
        lines.extend(print_indent(&self.env_var.user_readable()?, "  ", 2));
        // #[cfg(target_os = "macos")]
        // lines.push("- integrations:".into());
        // #[cfg(target_os = "macos")]
        // if let Some(integrations) = &self.integrations {
        // lines.extend(print_indent(&integrations.user_readable()?, "  ", 1));
        // }
        Ok(lines)
    }
}
