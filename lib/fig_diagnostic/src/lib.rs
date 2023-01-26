use std::fmt::Display;

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
use time::format_description::well_known::Rfc3339;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub model_name: Option<String>,
    pub model_identifier: Option<String>,
    pub chip: Option<String>,
    pub total_cores: Option<String>,
    pub memory: Option<String>,
}

impl HardwareInfo {
    fn new() -> HardwareInfo {
        use sysinfo::{
            CpuExt,
            System,
            SystemExt,
        };

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

        hardware_info
    }

    fn user_readable(&self) -> Vec<String> {
        vec![
            format!("model: {}", self.model_name.as_deref().unwrap_or_default()),
            format!("model-id: {}", self.model_identifier.as_deref().unwrap_or_default()),
            format!("chip-id: {}", self.chip.as_deref().unwrap_or_default()),
            format!("cores: {}", self.total_cores.as_deref().unwrap_or_default()),
            format!("mem: {}", self.memory.as_deref().unwrap_or_default()),
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVarDiagnostic {
    pub env_vars: Vec<(String, String)>,
}

impl EnvVarDiagnostic {
    fn new() -> EnvVarDiagnostic {
        let env_vars = std::env::vars()
            .filter(|(key, val)| {
                (key.starts_with("FIG_")
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
                    || key == "__CFBundleIdentifier"
                    || key == "ZDOTDIR")
                    && !key.contains("TOKEN")
                    && !key.contains("KEY")
                    && !key.contains("SECRET")
                    && !key.contains("PASSWORD")
                    && !val.starts_with("figapi_")
            })
            .collect();

        EnvVarDiagnostic { env_vars }
    }

    fn user_readable(&self) -> Vec<String> {
        let mut lines = vec![];
        for (key, value) in &self.env_vars {
            lines.push(format!("{key}: {value}"));
        }

        lines
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentEnvironment {
    pub shell_exe: Option<String>,
    pub figterm_exe: Option<String>,
    pub terminal_exe: Option<String>,
    pub current_dir: String,
    pub executable_location: String,
    pub terminal: Option<Terminal>,
    pub install_method: InstallMethod,
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

    fn user_readable(&self) -> Vec<String> {
        vec![
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
        ]
    }
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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DotfilesDiagnostics {
    pub profile: Option<String>,
    pub bashrc: Option<String>,
    pub bash_profile: Option<String>,
    pub zshrc: Option<String>,
    pub zprofile: Option<String>,
}

impl DotfilesDiagnostics {
    fn new() -> DotfilesDiagnostics {
        let home_dir = match directories::home_dir() {
            Ok(home_dir) => home_dir,
            Err(_) => return DotfilesDiagnostics::default(),
        };

        let profile = std::fs::read_to_string(home_dir.join(".profile")).ok();
        let bashrc = std::fs::read_to_string(home_dir.join(".bashrc")).ok();
        let bash_profile = std::fs::read_to_string(home_dir.join(".bash_profile")).ok();
        let zshrc = std::fs::read_to_string(home_dir.join(".zshrc")).ok();
        let zprofile = std::fs::read_to_string(home_dir.join(".zprofile")).ok();

        DotfilesDiagnostics {
            profile,
            bashrc,
            bash_profile,
            zshrc,
            zprofile,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Diagnostics {
    pub time: String,
    pub version: String,
    pub hardware: HardwareInfo,
    pub os: Option<OSVersion>,
    pub user_env: CurrentEnvironment,
    pub env_var: EnvVarDiagnostic,
    pub dotfiles: DotfilesDiagnostics,
}

impl Diagnostics {
    pub fn new() -> Diagnostics {
        let time = time::OffsetDateTime::now_utc().format(&Rfc3339).unwrap_or_default();

        Diagnostics {
            time,
            version: env!("CARGO_PKG_VERSION").to_owned(),
            hardware: HardwareInfo::new(),
            os: fig_util::system_info::os_version().cloned(),
            user_env: CurrentEnvironment::new(),
            env_var: EnvVarDiagnostic::new(),
            dotfiles: DotfilesDiagnostics::new(),
        }
    }

    pub fn user_readable(&self) -> Vec<String> {
        let print_indent = |lines: &[String], indent: &str, level: usize| {
            let mut new_lines = vec![];
            for line in lines {
                new_lines.push(format!("{}- {}", indent.repeat(level), line));
            }
            new_lines
        };

        let mut lines = vec![];

        lines.push("fig-details:".into());
        lines.extend(print_indent(&[self.version.to_owned()], "  ", 1));
        lines.push("hardware-info:".into());
        lines.extend(print_indent(&self.hardware.user_readable(), "  ", 1));
        lines.push("os-info:".into());
        match self.os {
            Some(ref os) => lines.extend(print_indent(&os.user_readable(), "  ", 1)),
            None => lines.push(format!("  - os: {}", std::env::consts::OS)),
        }
        lines.push("environment:".into());
        lines.extend(print_indent(&self.user_env.user_readable(), "  ", 1));
        lines.push("  - env-vars:".into());
        lines.extend(print_indent(&self.env_var.user_readable(), "  ", 2));

        lines
    }
}

impl Default for Diagnostics {
    fn default() -> Self {
        Self::new()
    }
}

// mod legacy_diagnostics {
//    TODO(sean) add back integration information once we have a better
//    understanding of IME/terminal integrations.
//    let mut integrations = IntegrationDiagnostics::new().await;
//    integrations.docker(&diagnostics.docker);
//
//    #[cfg(target_os = "macos")]
//    lines.push("- integrations:".into());
//    #[cfg(target_os = "macos")]
//    if let Some(integrations) = &self.integrations {
//    lines.extend(print_indent(&integrations.user_readable()?, "  ", 1));
//    }
//
//    #[derive(Debug, Clone, Serialize, Deserialize)]
//    struct IntegrationDiagnostics {
//        integrations: Vec<(Integrations, IntegrationStatus)>,
//    }
//
//    #[allow(dead_code)]
//    impl IntegrationDiagnostics {
//        #[cfg(target_os = "macos")]
//        async fn new() -> IntegrationDiagnostics {
//            let mut integrations = vec![
//                (Integrations::Ssh, IntegrationStatus { status: "false".into() }),
//                (Integrations::Tmux, IntegrationStatus { status: "false".into() }),
//            ];
//
//            let integration_result = verify_integration("com.googlecode.iterm2")
//                .await
//                .unwrap_or_else(|e| format!("Error {}", e));
//
//            integrations.push((Integrations::ITerm, IntegrationStatus {
//                status: integration_result,
//            }));
//
//            let integration_result = verify_integration("co.zeit.hyper")
//                .await
//                .unwrap_or_else(|e| format!("Error {}", e));
//
//            integrations.push((Integrations::Hyper, IntegrationStatus {
//                status: integration_result,
//            }));
//
//            let integration_result = verify_integration("com.microsoft.VSCode")
//                .await
//                .unwrap_or_else(|e| format!("Error {}", e));
//
//            integrations.push((Integrations::VsCode, IntegrationStatus {
//                status: integration_result,
//            }));
//
//            IntegrationDiagnostics { integrations }
//        }
//
//        #[cfg(target_os = "macos")]
//        fn docker(&mut self, status: impl Into<String>) {
//            self.integrations
//                .push((Integrations::Docker, IntegrationStatus { status: status.into() }));
//        }
//
//        fn user_readable(&self) -> Vec<String> {
//            let mut lines = vec![];
//            for (integration, status) in &self.integrations {
//                lines.push(format!("{}: {}", integration, status.status));
//            }
//
//            lines
//        }
//    }
//
//    pub async fn verify_integration(integration: impl Into<String>) -> Result<String> {
//        let response =
// send_recv_command_to_socket(command::Command::TerminalIntegration(TerminalIntegrationCommand {
//            identifier: integration.into(),
//            action: IntegrationAction::VerifyInstall as i32,
//        }))
//        .await?
//        .context("Received EOF while getting terminal integration")?;
//
//        let message = match response.response {
//            Some(Response::Success(success)) => success.message,
//            Some(Response::Error(error)) => error.message,
//            _ => eyre::bail!("Invalid response"),
//        };
//
//        message.context("No message found")
//    }
//}
