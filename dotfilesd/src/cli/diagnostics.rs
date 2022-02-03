use std::{time::Duration};
use std::path::PathBuf;
use std::fs;
use regex::Regex;
use std::process::Command;
use anyhow::{Context, Result};
use directories::BaseDirs;
use super::util::{get_os_version};
use crate::util::glob;
use crate::proto::local::{
    command,
    command_response::{Response},
    DiagnosticsCommand,
    DiagnosticsResponse,
    TerminalIntegrationCommand,
    IntegrationAction
};
use crate::ipc::{
    command::send_recv_command,
    connect_timeout,
    get_socket_path
};

fn home_dir() -> Result<String> {
    let home = BaseDirs::new()
        .context("Could not get home dir")?
        .home_dir()
        .to_string_lossy()
        .into_owned();

    Ok(home)
}

pub fn dscl_read(value: &str) -> Result<String> {
    let result = Command::new("dscl")
        .arg(".")
        .arg("-read")
        .arg(home_dir()?)
        .arg(value)
        .output()
        .with_context(|| "Could not read value")?;

    Ok(String::from_utf8_lossy(&result.stdout).trim().to_string())
}

fn load_settings() -> Result<serde_json::Value> {
    let settings_path = BaseDirs::new()
        .context("Could not get home dir")?
        .home_dir()
        .join(".fig")
        .join("settings.json");

    let settings_file = fs::read_to_string(settings_path)?;

    Ok(serde_json::from_str(&settings_file)?)
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

    let mut specs: Vec<PathBuf> = vec![];
    for entry in fs::read_dir(specs_location)? {
        if let Ok(entry) = entry {
            let path = entry.path();
            if glob.is_match(&path) {
                specs.push(path);
            }
        }
    }

    Ok(specs)
}

fn match_regex(regex: &str, input: &str) -> Option<String> {
    Some(Regex::new(regex).unwrap().captures(input)?.get(1)?.as_str().to_string())
}

struct HardwareInfo {
    model_name: Option<String>,
    model_identifier: Option<String>,
    chip: Option<String>,
    total_cores: Option<String>,
    memory: Option<String>
}

fn get_hardware_diagnostics() -> Result<HardwareInfo> {
    let result = Command::new("system_profiler")
        .arg("SPHardwareDataType")
        .output()
        .with_context(|| "Could not read hardware")?;
    let text = String::from_utf8_lossy(&result.stdout).trim().to_string();

    Ok(HardwareInfo {
        model_name: match_regex(r"Model Name: (.+)", &text),
        model_identifier: match_regex(r"Model Identifier: (.+)", &text),
        chip: match_regex(r"Chip: (.+)", &text),
        total_cores: match_regex(r"Total Number of Cores: (.+)", &text),
        memory: match_regex(r"Memory: (.+)", &text),
    })
}

pub async fn verify_integration(integration: &str) -> Result<String> {
    let path = get_socket_path();
    let mut conn = connect_timeout(&path, Duration::from_secs(3)).await?;
    let response = send_recv_command(&mut conn, command::Command::TerminalIntegration(TerminalIntegrationCommand {
        identifier: integration.to_string(),
        action: IntegrationAction::VerifyInstall as i32
    })).await?;

    let message = match response.response {
        Some(Response::Success(success)) => success.message,
        Some(Response::Error(error)) => error.message,
        _ => anyhow::bail!("Invalid response")
    };

    message.context("No message found")
}

fn installed_via_brew() -> Result<bool> {
    let result = Command::new("brew")
        .arg("list")
        .arg("--cask")
        .output()
        .with_context(|| "Could not get brew casks")?;
    let text = String::from_utf8_lossy(&result.stdout).trim().to_string();

    Ok(Regex::new(r"(?m:^fig$)").unwrap().is_match(&text))
}

pub async fn get_diagnostics() -> Result<DiagnosticsResponse> {
    let path = get_socket_path();
    let mut conn = connect_timeout(&path, Duration::from_secs(3)).await?;
    let response = send_recv_command(&mut conn, command::Command::Diagnostics(DiagnosticsCommand {})).await?;

    match response.response {
        Some(Response::Diagnostics(diagnostics)) => Ok(diagnostics),
        _ => anyhow::bail!("Invalid response")
    }
}

pub async fn summary() -> Result<String> {
    let mut lines: Vec<String> = vec![];

    let diagnostics = get_diagnostics().await?;
    let mut version: Vec<&str> = vec![&diagnostics.distribution];

    if diagnostics.beta { version.push("[Beta]") }
    if diagnostics.debug_autocomplete { version.push("[Debug]") }
    if diagnostics.developer_mode_enabled { version.push("[Dev]") }

    let layout_name = format!("[{}]", diagnostics.current_layout_name);
    if layout_name != "[]" {
        version.push(&layout_name);
    };

    if diagnostics.is_running_on_read_only_volume {
        version.push("TRANSLOCATED!");
    }

    lines.push(format!("Fig Version: {}", version.join(" ")));
    lines.push(format!("{}", dscl_read("UserShell").unwrap_or("Unknown UserShell".to_string())));
    lines.push(format!("Bundle path: {}", diagnostics.path_to_bundle));

	  lines.push(format!("Autocomplete: {}", diagnostics.autocomplete));
		lines.push(format!("Settings.json: {}", load_settings().is_ok()));

    lines.push("CLI installed: true".to_string());

    let executable = std::env::current_exe()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or("<none>".to_string());

    lines.push(format!("CLI tool path: {}", executable));
    lines.push(format!("Accessibility: {}", diagnostics.accessibility));

    let num_specs = get_local_specs().map_or(0, |v| v.len());
    lines.push(format!("Number of specs: {}", num_specs));

    lines.push("SSH Integration: false".to_string());
    lines.push("Tmux Integration: false".to_string());
    lines.push(format!("Keybindings path: {}", diagnostics.keypath));

    let integration_result = verify_integration("com.googlecode.iterm2").await
        .unwrap_or_else(|e| format!("Error {}", e));
    lines.push(format!("iTerm Integration: {}", integration_result));

    let integration_result = verify_integration("co.zeit.hyper").await
        .unwrap_or_else(|e| format!("Error {}", e));
    lines.push(format!("Hyper Integration: {}", integration_result));

    let integration_result = verify_integration("com.microsoft.VSCode").await
        .unwrap_or_else(|e| format!("Error {}", e));
    lines.push(format!("VSCode Integration: {}", integration_result));

    lines.push(format!("Docker Integration: {}", diagnostics.docker));
    lines.push(format!("Symlinked dotfiles: {}", diagnostics.symlinked));
    lines.push(format!("Only insert on tab: {}", diagnostics.onlytab));

    if let Ok(true) = installed_via_brew() {
        lines.push("Installed via Brew: true".to_string());
    }

    lines.push(format!("Installation Script: {}", diagnostics.installscript));
    lines.push(format!("PseudoTerminal Path: {}", diagnostics.psudoterminal_path));
    lines.push(format!("SecureKeyboardInput: {}", diagnostics.securekeyboard));
    lines.push(format!("SecureKeyboardProcess: {}", diagnostics.securekeyboard_path));
    lines.push(format!("Current active process: {}", diagnostics.current_process));

    let cwd = std::env::current_dir()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or("Could not get working directory".to_string());
    lines.push(format!("Current working directory: {}", cwd));
    lines.push(format!("Current window identifier: {}", diagnostics.current_window_identifier));
    lines.push(format!("Path: {}", std::env::var("PATH").unwrap_or("Could not get path".to_string())));

    // Fig envs
    lines.push("Fig environment variables:".to_string());
    for (key, value) in std::env::vars() {
        if key.starts_with("FIG_") || key.eq("TERM_SESSION_ID") {
            lines.push(format!("  - {}={}", key, value));
        }
    }

    let version = get_os_version().map(|v| v.to_string());
    lines.push(format!("OS Version: {}", version.unwrap_or("Could not get OS Version".to_string())));

    // Hardware
    let hardware_diagnostics = get_hardware_diagnostics();
    if let Ok(HardwareInfo { model_name, model_identifier, chip, total_cores, memory }) = hardware_diagnostics {
        lines.push("Hardware:".to_string());
        lines.push(format!("  - Model Name: {}", model_name.unwrap_or("".to_string())));
        lines.push(format!("  - Model Identifier: {}", model_identifier.unwrap_or("".to_string())));
        lines.push(format!("  - Chip: {}", chip.unwrap_or("".to_string())));
        lines.push(format!("  - Cores: {}", total_cores.unwrap_or("".to_string())));
        lines.push(format!("  - Memory: {}", memory.unwrap_or("".to_string())));
    } else {
        lines.push("Could not get hardware information.".to_string());
    }

	  Ok(lines.join("\n"))
}

pub async fn diagnostics_cli() -> Result<()> {
    println!("{}", summary().await?);

    Ok(())
}
