pub mod backoff;
pub mod spinner;
pub mod sync;

use std::env;
use std::ffi::OsStr;
use std::path::{
    Path,
    PathBuf,
};
use std::process::Command;

use cfg_if::cfg_if;
use crossterm::style::Stylize;
use dialoguer::theme::ColorfulTheme;
use dialoguer::FuzzySelect;
use eyre::{
    bail,
    Result,
    WrapErr,
};
use globset::{
    Glob,
    GlobSet,
    GlobSetBuilder,
};
use once_cell::sync::Lazy;
use regex::Regex;
use tracing::warn;

#[derive(Debug)]
pub struct LaunchArgs {
    pub print_running: bool,
    pub print_launching: bool,
    pub wait_for_launch: bool,
}

#[must_use]
pub fn fig_bundle() -> Option<PathBuf> {
    cfg_if! {
        if #[cfg(target_os = "macos")] {
            Some(PathBuf::from("/Applications/Fig.app/"))
        } else {
            None
        }
    }
}

/// Glob patterns against full paths
pub fn glob_dir(glob: &GlobSet, directory: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    // List files in the directory
    let dir = std::fs::read_dir(directory)?;

    for entry in dir {
        let path = entry?.path();

        // Check if the file matches the glob pattern
        if glob.is_match(&path) {
            files.push(path);
        }
    }

    Ok(files)
}

/// Glob patterns against the file name
pub fn glob_files(glob: &GlobSet, directory: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    // List files in the directory
    let dir = std::fs::read_dir(directory)?;

    for entry in dir {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name();

        // Check if the file matches the glob pattern
        if let Some(file_name) = file_name {
            if glob.is_match(file_name) {
                files.push(path);
            }
        }
    }

    Ok(files)
}

pub fn glob<I, S>(patterns: I) -> Result<GlobSet>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern.as_ref())?);
    }
    Ok(builder.build()?)
}

pub fn app_path_from_bundle_id(bundle_id: impl AsRef<OsStr>) -> Option<String> {
    cfg_if! {
        if #[cfg(target_os = "macos")] {
            let installed_apps = std::process::Command::new("mdfind")
                .arg("kMDItemCFBundleIdentifier")
                .arg("=")
                .arg(bundle_id)
                .output()
                .ok()?;

            let path = String::from_utf8_lossy(&installed_apps.stdout);
            Some(path.trim().split('\n').next()?.into())
        } else {
            let _bundle_id = bundle_id;
            None
        }
    }
}

#[must_use]
pub fn is_app_running() -> bool {
    cfg_if! {
        if #[cfg(target_os = "macos")] {
            let output = match std::process::Command::new("lsappinfo")
                .args(["info", "-app", "com.mschrage.fig"])
                .output()
            {
                Ok(output) => output,
                Err(_) => return false,
            };

            match std::str::from_utf8(&output.stdout) {
                Ok(result) => !result.trim().is_empty(),
                Err(_) => false,
            }
        } else {
            use sysinfo::{
                ProcessRefreshKind,
                RefreshKind,
                System,
                SystemExt,
            };

            cfg_if! {
                if #[cfg(target_os = "windows")] {
                    let process_name = "fig_desktop.exe";
                } else if #[cfg(target_os = "linux")] {
                    let process_name = match fig_util::wsl::is_wsl() {
                        true => {
                            let output = match std::process::Command::new("tasklist.exe").args(["/NH", "/FI", "IMAGENAME eq fig_desktop.exe"]).output() {
                                Ok(output) => output,
                                Err(_) => return false,
                            };

                            return match std::str::from_utf8(&output.stdout) {
                                Ok(result) => result.contains("fig_desktop.exe"),
                                Err(_) => false,
                            };
                        },
                        false => "fig_desktop",
                    };
                }
            }

            let s = System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()));
            let mut processes = s.processes_by_exact_name(process_name);
            processes.next().is_some()
        }
    }
}

pub fn launch_fig(args: LaunchArgs) -> Result<()> {
    use fig_util::directories::fig_socket_path;

    if is_app_running() {
        if args.print_running {
            println!("Fig is already running");
        }

        return Ok(());
    }

    if args.print_launching {
        println!("Launching Fig");
    }

    std::fs::remove_file(fig_socket_path()?).ok();

    cfg_if! {
        if #[cfg(target_os = "linux")] {
            if fig_util::wsl::is_wsl() {
                let output = Command::new("fig_desktop.exe")
                    .output()
                    .context("Unable to launch Fig")?;

                if !output.status.success() {
                    bail!("Failed to launch Fig: {}", String::from_utf8_lossy(&output.stderr));
                }
            } else {
                let output = Command::new("systemctl")
                    .args(&["--user", "start", "fig"])
                    .output()
                    .context("Unable to launch Fig")?;

                if !output.status.success() {
                    bail!("Failed to launch Fig: {}", String::from_utf8_lossy(&output.stderr));
                }
            }
        } else if #[cfg(target_os = "macos")] {
            Command::new("open")
                .args(["-g", "-b", "com.mschrage.fig"])
                .output()
                .context("Unable to launch Fig")?;
        } else if #[cfg(target_os = "windows")] {
            use std::os::windows::process::CommandExt;
            use windows::Win32::System::Threading::DETACHED_PROCESS;

            Command::new("fig_desktop")
                .creation_flags(DETACHED_PROCESS.0)
                .spawn()
                .context("Unable to launch Fig")?;
        }
    }

    if !args.wait_for_launch {
        return Ok(());
    }

    #[cfg(not(target_os = "windows"))]
    if !is_app_running() {
        eyre::bail!("Unable to launch Fig");
    }

    // Wait for socket to exist
    let path = fig_socket_path()?;

    cfg_if! {
        if #[cfg(not(target_os = "windows"))] {
            for _ in 0..10 {
                // Wait for socket to exist
                if path.exists() {
                    return Ok(());
                }

                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        } else if #[cfg(target_os = "windows")] {
            for _ in 0..20 {
                match path.metadata() {
                    Ok(_) => return Ok(()),
                    Err(err) => if let Some(code) = err.raw_os_error() {
                        // Windows can't query socket file existence
                        // Check against arbitrary error code
                        if code == 1920 {
                            return Ok(())
                        }
                    },
                }

                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }
    }

    bail!("Unable to finish launching Fig properly")
}

pub fn is_executable_in_path(program: impl AsRef<Path>) -> bool {
    match env::var_os("PATH") {
        Some(path) => env::split_paths(&path).any(|p| p.join(&program).is_file()),
        _ => false,
    }
}

pub fn app_not_running_message() -> String {
    format!(
        "\n{}\nFig might not be running, to launch Fig run: {}\n",
        "Unable to connect to Fig".bold(),
        "fig launch".magenta()
    )
}

pub fn login_message() -> String {
    format!(
        "{}\nLooks like you aren't logged in to fig, to login run: {}",
        "Not logged in".bold(),
        "fig login".magenta()
    )
}

pub fn get_fig_version() -> Result<String> {
    cfg_if! {
        if #[cfg(target_os = "macos")] {
            use eyre::ContextCompat;
            use regex::Regex;

            let plist = std::fs::read_to_string("/Applications/Fig.app/Contents/Info.plist")?;

            let get_plist_field = |field: &str| -> Result<String> {
                let regex =
                    Regex::new(&format!("<key>{}</key>\\s*<\\S+>(\\S+)</\\S+>", field)).unwrap();
                let value = regex
                    .captures(&plist)
                    .context(format!("Could not find {} in plist", field))?
                    .get(1)
                    .context(format!("Could not find {} in plist", field))?
                    .as_str();

                Ok(value.into())
            };

            let fig_version = get_plist_field("CFBundleShortVersionString")?;
           Ok(fig_version)
        } else {
            use std::process::Command;
            Ok(String::from_utf8_lossy(&Command::new("fig_desktop").arg("--version").output()?.stdout)
                .replace("fig_desktop", "").trim().into())
        }
    }
}

pub fn dialoguer_theme() -> ColorfulTheme {
    ColorfulTheme {
        prompt_prefix: dialoguer::console::style("?".into()).for_stderr().magenta(),
        ..ColorfulTheme::default()
    }
}

pub fn match_regex(regex: impl AsRef<str>, input: impl AsRef<str>) -> Option<String> {
    Some(
        Regex::new(regex.as_ref())
            .unwrap()
            .captures(input.as_ref())?
            .get(1)?
            .as_str()
            .into(),
    )
}

static IS_TTY: Lazy<bool> = Lazy::new(|| std::env::var("TTY").is_ok());

pub fn choose(prompt: &str, options: Vec<String>) -> Result<usize> {
    if options.is_empty() {
        bail!("no options passed to choose")
    }

    if !*IS_TTY {
        warn!("choose called without TTY, choosing first option");
        return Ok(0);
    }

    Ok(FuzzySelect::with_theme(&dialoguer_theme())
        .items(&options)
        .default(0)
        .with_prompt(prompt)
        .interact()?)
}

#[ignore]
#[test]
fn test() {
    use sysinfo::{
        ProcessRefreshKind,
        RefreshKind,
        System,
        SystemExt,
    };

    let s = System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()));
    cfg_if! {
        if #[cfg(windows)] {
            let mut processes = s.processes_by_name("fig_desktop");
            assert!(processes.next().is_some());
        } else {
            let mut processes = s.processes_by_exact_name("fig_desktop");
            assert!(processes.next().is_some());
        }
    }
}
