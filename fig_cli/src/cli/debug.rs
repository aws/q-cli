use std::path::Path;
use std::process::Command;

use anyhow::{
    anyhow,
    Context,
    Result,
};
use clap::{
    Subcommand,
    ValueEnum,
};
use crossterm::style::Stylize;
use fig_ipc::command::{
    input_method_command,
    prompt_accessibility_command,
    run_build_command,
    set_debug_mode,
    toggle_debug_mode,
};
use fig_proto::local::InputMethodAction;
use serde_json::json;

use crate::cli::app::quit_fig;
use crate::cli::diagnostics::get_diagnostics;
use crate::cli::launch_fig;
use crate::dotfiles::download_and_notify;
use crate::util::{
    glob,
    glob_dir,
    LaunchOptions,
};

#[derive(Debug, ValueEnum, Clone)]
pub enum Build {
    Dev,
    Prod,
    Staging,
}

#[derive(Debug, ValueEnum, Clone)]
pub enum ImeCommand {
    Install,
    Uninstall,
    Select,
    Deselect,
    Enable,
    Disable,
    Status,
    Register,
}

#[derive(Debug, ValueEnum, Clone)]
pub enum AutocompleteWindowDebug {
    On,
    Off,
}

#[derive(Debug, ValueEnum, Clone)]
pub enum AccessibilityAction {
    Refresh,
    Reset,
    Prompt,
    Open,
    Status,
}

#[derive(Debug, Subcommand)]
pub enum DebugSubcommand {
    /// Debug fig app
    App,
    /// Debug dotfiles
    Dotfiles {
        /// Disable debug mode
        #[clap(long, action)]
        disable: bool,
    },
    /// Switch build
    Build {
        #[clap(value_enum, value_parser)]
        build: Build,
    },
    /// Toggle/set autocomplete window debug mode
    AutocompleteWindow {
        #[clap(value_enum, value_parser)]
        mode: Option<AutocompleteWindowDebug>,
    },
    /// Show fig debug logs
    Logs {
        #[clap(long, value_parser)]
        files: Vec<String>,
    },
    /// Fig input method editor
    Ime {
        #[clap(value_enum, value_parser)]
        command: ImeCommand,
    },
    /// Prompt accessibility
    PromptAccessibility,
    /// Sample fig process
    Sample,
    /// Debug fig unix sockets
    UnixSocket,
    /// Debug fig codesign verification
    VerifyCodesign,

    ///
    Accessibility {
        #[clap(value_enum, value_parser)]
        action: Option<AccessibilityAction>,
    },
}

fn get_running_app_info(bundle_id: impl AsRef<str>, field: impl AsRef<str>) -> Result<String> {
    let info = Command::new("lsappinfo")
        .args(["info", "-only", field.as_ref(), "-app", bundle_id.as_ref()])
        .output()?;
    let info = String::from_utf8(info.stdout)?;
    let value = info
        .split('=')
        .nth(1)
        .context(anyhow!("Could not get field value for {}", field.as_ref()))?
        .replace('"', "");
    Ok(value.trim().into())
}

pub fn get_app_info() -> Result<String> {
    let output = Command::new("lsappinfo")
        .args(["info", "-app", "com.mschrage.fig"])
        .output()?;
    let result = String::from_utf8(output.stdout)?;
    Ok(result.trim().into())
}

impl DebugSubcommand {
    pub async fn execute(&self) -> Result<()> {
        match self {
            DebugSubcommand::App => {
                let app_info = get_app_info().unwrap_or_else(|_| "".into());
                if app_info.is_empty() {
                    println!("Fig app is not currently running. Attempting to start...");
                    if Command::new("open")
                        .args(["-g", "-b", "com.mschrage.fig"])
                        .spawn()?
                        .wait()
                        .is_err()
                    {
                        println!("Could not start fig");
                        return Ok(());
                    }
                }
                let fig_path = get_running_app_info("com.mschrage.fig", "bundlepath")?;
                let front_app = Command::new("lsappinfo").arg("front").output()?;
                let terminal_name = String::from_utf8(front_app.stdout)
                    .ok()
                    .and_then(|app| get_running_app_info(app, "name").ok());
                let terminal_text = match terminal_name {
                    Some(terminal) => format!(" ({})", terminal),
                    None => "".into(),
                };

                println!("Running the Fig.app executable directly from {}.", fig_path);
                println!(
                    "You will need to grant accessibility permissions to the current terminal{}!",
                    terminal_text
                );

                Command::new(format!("{}/Contents/MacOS/fig", fig_path))
                    .spawn()?
                    .wait()?;
            },
            DebugSubcommand::Build { build } => {
                let x = build.to_possible_value().context(anyhow!("Invalid build value"))?;
                let res = run_build_command(x.get_name()).await;
                if res.is_err() {
                    println!("\n{}", "Unable to connect to Fig.".bold());
                    println!(
                        "\nFig might not be running, to launch Fig run: {}\n",
                        "fig launch".magenta()
                    );
                    return res;
                }
            },
            DebugSubcommand::Dotfiles { disable } => {
                if *disable {
                    fig_settings::state::remove_value("developer.dotfiles.debug")?;
                } else {
                    fig_settings::state::set_value("developer.dotfiles.debug", json!(true))?;
                }
                download_and_notify().await.context("Could not sync remote dotfiles")?;
            },
            DebugSubcommand::AutocompleteWindow { mode } => {
                let result = match mode {
                    Some(AutocompleteWindowDebug::On) => set_debug_mode(true).await,
                    Some(AutocompleteWindowDebug::Off) => set_debug_mode(false).await,
                    None => toggle_debug_mode().await,
                };
                if result.is_err() {
                    println!("Could not update debug mode");
                    return result.map(|_| ());
                }
            },
            DebugSubcommand::Logs { files } => {
                fig_settings::state::set_value("developer.logging", json!(true))?;

                ctrlc::set_handler(|| {
                    let code = match fig_settings::state::set_value("developer.logging", json!(false)) {
                        Ok(_) => 0,
                        Err(_) => 1,
                    };
                    std::process::exit(code);
                })?;

                let log_dir = fig_directories::fig_dir()
                    .context("Could not find fig dir")?
                    .join("logs");

                let mut files = files.clone();

                let log_paths = if files.is_empty() {
                    let pattern = log_dir.join("*.log");
                    let globset = glob(&[pattern.to_str().unwrap()])?;
                    glob_dir(&globset, &log_dir)?
                } else {
                    let mut paths = Vec::new();

                    if files.iter().any(|f| f == "figterm") {
                        // Remove figterm from the list of files to open
                        files.retain(|f| f != "figterm");

                        // Add figterm*.log to the list of files to open
                        let pattern = log_dir.join("figterm*.log");
                        let globset = glob(&[pattern.to_str().unwrap()])?;
                        let figterm_logs = glob_dir(&globset, &log_dir)?;
                        paths.extend(figterm_logs);
                    }

                    // Push any remaining files to open
                    paths.extend(files.iter().map(|file| log_dir.join(format!("{}.log", file))));

                    paths
                };

                Command::new("tail")
                    .arg("-n0")
                    .arg("-qf")
                    .args(log_paths)
                    .spawn()?
                    .wait()?;
            },
            DebugSubcommand::Ime { command } => {
                let action = match command {
                    ImeCommand::Install => InputMethodAction::InstallInputMethod,
                    ImeCommand::Uninstall => InputMethodAction::UninstallInputMethod,
                    ImeCommand::Select => InputMethodAction::SelectInputMethod,
                    ImeCommand::Deselect => InputMethodAction::DeselectInputMethod,
                    ImeCommand::Enable => InputMethodAction::EnableInputMethod,
                    ImeCommand::Disable => InputMethodAction::DisableInputMethod,
                    ImeCommand::Status => InputMethodAction::StatusOfInputMethod,
                    ImeCommand::Register => InputMethodAction::RegisterInputMethod,
                };
                let result = input_method_command(action).await;
                if result.is_err() {
                    println!("Could not run ime command.");
                    return result;
                }
            },
            DebugSubcommand::PromptAccessibility => {
                let result = prompt_accessibility_command().await;
                if result.is_err() {
                    println!("Could not prompt for accessibility permissions.");
                    return result;
                }
            },
            DebugSubcommand::Sample => {
                let output = Command::new("lsappinfo")
                    .args(["info", "-only", "-pid", "-app", "com.mschrage.fig"])
                    .output()?;
                let pid_str = String::from_utf8(output.stdout)?;
                let pid = pid_str.split('=').nth(1).context("Could not get Fig app pid")?.trim();
                let outfile = Path::new("/tmp").join("fig-sample");

                println!(
                    "Sampling Fig process ({}). Writing output to {}",
                    pid,
                    outfile.display()
                );
                let result = Command::new("sample")
                    .arg("-f")
                    .arg::<&Path>(outfile.as_ref())
                    .arg(pid)
                    .spawn()?
                    .wait();
                if result.is_err() {
                    println!("Could not sample Fig process.");
                    anyhow::bail!("Failed to sample Fig process.");
                }
                println!("\n\n\n-------\nFinished writing to {}", outfile.display());
                println!("Please send this file to the Fig Team");
                println!("Or attach it to a Github issue (run '{}')", "fig issue".magenta());
            },
            DebugSubcommand::UnixSocket => {
                println!("Listening on /tmp/fig.socket...");
                println!("Note: You will need to restart Fig afterwards");
                let socket_path = "/tmp/fig.socket";
                std::fs::remove_file(socket_path)?;
                Command::new("nc").args(["-Ulk", socket_path]).spawn()?.wait()?;
            },
            DebugSubcommand::VerifyCodesign => {
                Command::new("codesign")
                    .args(["-vvvv", "/Applications/Fig.app"])
                    .spawn()?
                    .wait()?;
            },
            DebugSubcommand::Accessibility { action } => match action {
                Some(AccessibilityAction::Refresh) => {
                    quit_fig().await?;

                    Command::new("tccutil")
                        .args(["reset", "Accessibility", "com.mschrage.fig"])
                        .spawn()?
                        .wait()?;

                    launch_fig(LaunchOptions::new().wait_for_activation().verbose())?;
                    // let result = prompt_accessibility_command().await;
                    // if result.is_err() {
                    //     println!("Could not prompt for accessibility permissions.");
                    //     return result;
                    // }
                },
                Some(AccessibilityAction::Reset) => {
                    quit_fig().await?;

                    Command::new("tccutil")
                        .args(["reset", "Accessibility", "com.mschrage.fig"])
                        .spawn()?
                        .wait()?;
                },
                Some(AccessibilityAction::Prompt) => {
                    launch_fig(LaunchOptions::new().wait_for_activation().verbose())?;
                    let result = prompt_accessibility_command().await;
                    if result.is_err() {
                        println!("Could not prompt for accessibility permissions.");
                        return result;
                    }
                },
                Some(AccessibilityAction::Open) => {
                    Command::new("open")
                        .args(["x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"])
                        .spawn()?
                        .wait()?;
                },
                Some(AccessibilityAction::Status) | None => {
                    let diagnostic = get_diagnostics().await?;

                    println!("Accessibility Enabled: {}", diagnostic.accessibility)
                },
            },
        }
        Ok(())
    }
}
