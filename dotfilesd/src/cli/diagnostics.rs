use super::util::get_os_version;
use crate::ipc::{command::send_recv_command, connect_timeout, get_socket_path};
use crate::proto::local::{
    command, command_response::Response, DiagnosticsCommand, DiagnosticsResponse,
    IntegrationAction, TerminalIntegrationCommand,
};
use crate::util::{glob, glob_dir, home_dir, Settings};
use anyhow::{Context, Result};
use directories::BaseDirs;
use regex::Regex;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

pub fn dscl_read(value: impl AsRef<OsStr>) -> Result<String> {
    let result = Command::new("dscl")
        .arg(".")
        .arg("-read")
        .arg(home_dir().context("Could not get home dir")?)
        .arg(value)
        .output()
        .context("Could not read value")?;

    Ok(String::from_utf8_lossy(&result.stdout).trim().into())
}

fn get_local_specs() -> Result<Vec<PathBuf>> {
    let specs_location = BaseDirs::new()
        .context("Could not get home dir")?
        .home_dir()
        .join(".fig")
        .join("autocomplete");
    let glob_pattern = specs_location.join("*.js");
    let patterns = [glob_pattern.to_str().unwrap()];
    let glob = glob(&patterns)?;

    glob_dir(&glob, specs_location)
}

fn match_regex(regex: impl AsRef<str>, input: impl AsRef<str>) -> Option<String> {
    Some(
        Regex::new(regex.as_ref())
            .unwrap()
            .captures(input.as_ref())?
            .get(1)?
            .as_str()
            .into(),
    )
}

struct HardwareInfo {
    model_name: Option<String>,
    model_identifier: Option<String>,
    chip: Option<String>,
    total_cores: Option<String>,
    memory: Option<String>,
}

fn get_hardware_diagnostics() -> Result<HardwareInfo> {
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
}

pub async fn verify_integration(integration: impl Into<String>) -> Result<String> {
    let path = get_socket_path();
    let mut conn = connect_timeout(&path, Duration::from_secs(3)).await?;
    let response = send_recv_command(
        &mut conn,
        command::Command::TerminalIntegration(TerminalIntegrationCommand {
            identifier: integration.into(),
            action: IntegrationAction::VerifyInstall as i32,
        }),
    )
    .await?;

    let message = match response.response {
        Some(Response::Success(success)) => success.message,
        Some(Response::Error(error)) => error.message,
        _ => anyhow::bail!("Invalid response"),
    };

    message.context("No message found")
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
    let path = get_socket_path();
    let mut conn = connect_timeout(&path, Duration::from_secs(3)).await?;
    let response = send_recv_command(
        &mut conn,
        command::Command::Diagnostics(DiagnosticsCommand {}),
    )
    .await?;

    match response.response {
        Some(Response::Diagnostics(diagnostics)) => Ok(diagnostics),
        _ => anyhow::bail!("Invalid response"),
    }
}

pub async fn summary() -> Result<String> {
    let mut lines: Vec<String> = vec![];

    let diagnostics = get_diagnostics().await?;
    let mut version: Vec<&str> = vec![&diagnostics.distribution];

    if diagnostics.beta {
        version.push("[Beta]")
    }
    if diagnostics.debug_autocomplete {
        version.push("[Debug]")
    }
    if diagnostics.developer_mode_enabled {
        version.push("[Dev]")
    }

    let layout_name = format!("[{}]", diagnostics.current_layout_name);
    if layout_name != "[]" {
        version.push(&layout_name);
    };

    if diagnostics.is_running_on_read_only_volume {
        version.push("TRANSLOCATED!");
    }

    lines.push(format!("Fig Version: {}", version.join(" ")));
    lines.push(dscl_read("UserShell").unwrap_or_else(|_| "Unknown UserShell".into()));
    lines.push(format!("Bundle path: {}", diagnostics.path_to_bundle));

    lines.push(format!("Autocomplete: {}", diagnostics.autocomplete));
    lines.push(format!("Settings.json: {}", Settings::load().is_ok()));

    lines.push("CLI installed: true".into());

    let executable = std::env::current_exe()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "<none>".into());

    lines.push(format!("CLI tool path: {}", executable));
    lines.push(format!("Accessibility: {}", diagnostics.accessibility));

    let num_specs = get_local_specs().map_or(0, |v| v.len());
    lines.push(format!("Number of specs: {}", num_specs));

    lines.push("SSH Integration: false".into());
    lines.push("Tmux Integration: false".into());
    lines.push(format!("Keybindings path: {}", diagnostics.keypath));

    let integration_result = verify_integration("com.googlecode.iterm2")
        .await
        .unwrap_or_else(|e| format!("Error {}", e));
    lines.push(format!("iTerm Integration: {}", integration_result));

    let integration_result = verify_integration("co.zeit.hyper")
        .await
        .unwrap_or_else(|e| format!("Error {}", e));
    lines.push(format!("Hyper Integration: {}", integration_result));

    let integration_result = verify_integration("com.microsoft.VSCode")
        .await
        .unwrap_or_else(|e| format!("Error {}", e));
    lines.push(format!("VSCode Integration: {}", integration_result));

    lines.push(format!("Docker Integration: {}", diagnostics.docker));
    lines.push(format!("Symlinked dotfiles: {}", diagnostics.symlinked));
    lines.push(format!("Only insert on tab: {}", diagnostics.onlytab));

    if let Ok(true) = installed_via_brew() {
        lines.push("Installed via Brew: true".into());
    }

    lines.push(format!(
        "Installation Script: {}",
        diagnostics.installscript
    ));
    lines.push(format!(
        "PseudoTerminal Path: {}",
        diagnostics.psudoterminal_path
    ));
    lines.push(format!(
        "SecureKeyboardInput: {}",
        diagnostics.securekeyboard
    ));
    lines.push(format!(
        "SecureKeyboardProcess: {}",
        diagnostics.securekeyboard_path
    ));
    lines.push(format!(
        "Current active process: {}",
        diagnostics.current_process
    ));

    let cwd = std::env::current_dir()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "Could not get working directory".into());
    lines.push(format!("Current working directory: {}", cwd));
    lines.push(format!(
        "Current window identifier: {}",
        diagnostics.current_window_identifier
    ));
    lines.push(format!(
        "Path: {}",
        std::env::var("PATH").unwrap_or_else(|_| "Could not get path".into())
    ));

    // Fig envs
    lines.push("Fig environment variables:".into());
    for (key, value) in std::env::vars() {
        if key.starts_with("FIG_") || key == "TERM_SESSION_ID" {
            lines.push(format!("  - {}={}", key, value));
        }
    }

    let version: String = get_os_version()
        .map(|v| v.into())
        .unwrap_or_else(|_| "Could not get OS Version".into());
    lines.push(format!("OS Version: {}", version));

    // Hardware
    let hardware_diagnostics = get_hardware_diagnostics();
    if let Ok(HardwareInfo {
        model_name,
        model_identifier,
        chip,
        total_cores,
        memory,
    }) = hardware_diagnostics
    {
        lines.push("Hardware:".into());
        lines.push(format!(
            "  - Model Name: {}",
            model_name.unwrap_or_default()
        ));
        lines.push(format!(
            "  - Model Identifier: {}",
            model_identifier.unwrap_or_default()
        ));
        lines.push(format!("  - Chip: {}", chip.unwrap_or_default()));
        lines.push(format!("  - Cores: {}", total_cores.unwrap_or_default()));
        lines.push(format!("  - Memory: {}", memory.unwrap_or_default()));
    } else {
        lines.push("Could not get hardware information.".into());
    }

    Ok(lines.join("\n"))
}

pub async fn diagnostics_cli() -> Result<()> {
    println!("{}", summary().await?);

    Ok(())
}
