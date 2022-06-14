pub mod api;
pub mod backoff;
pub mod os_version;
pub mod sync;

use std::env;
use std::ffi::OsStr;
use std::path::{
    Path,
    PathBuf,
};

use anyhow::{
    bail,
    Result,
};
use cfg_if::cfg_if;
use crossterm::style::Stylize;
use dialoguer::theme::ColorfulTheme;
use globset::{
    Glob,
    GlobSet,
    GlobSetBuilder,
};
pub use os_version::{
    OSVersion,
    SupportLevel,
};
use regex::Regex;

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
        } else if #[cfg(target_os = "linux")] {
            use sysinfo::{
                ProcessRefreshKind,
                RefreshKind,
                System,
                SystemExt,
            };

            let s = System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()));
            let mut processes = s.processes_by_exact_name("fig_desktop");
            processes.next().is_some()
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[must_use]
pub struct LaunchOptions {
    pub wait_for_activation: bool,
    pub verbose: bool,
}

impl LaunchOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn wait_for_activation(self) -> Self {
        Self {
            wait_for_activation: true,
            ..self
        }
    }

    pub fn verbose(self) -> Self {
        Self { verbose: true, ..self }
    }
}

pub fn launch_fig(opts: LaunchOptions) -> Result<()> {
    cfg_if! {
        if #[cfg(target_os = "macos")] {
            use anyhow::Context;
            use fig_ipc::get_fig_socket_path;

            if is_app_running() {
                return Ok(());
            }

            if opts.verbose {
                println!("\n→ Launching Fig...\n");
            }

            std::process::Command::new("open")
                .args(["-g", "-b", "com.mschrage.fig"])
                .output()
                .context("\nUnable to launch Fig\n")?;

            if !opts.wait_for_activation {
                return Ok(());
            }

            if !is_app_running() {
                anyhow::bail!("Unable to launch Fig");
            }

            // Wait for socket to exist
            let path = get_fig_socket_path();
            for _ in 0..9 {
                if path.exists() {
                    return Ok(());
                }
                // Sleep for a bit
                std::thread::sleep(std::time::Duration::from_millis(500));
            }

            bail!("\nUnable to finish launching Fig properly\n")
        } else if #[cfg(target_os = "linux")] {
            use anyhow::Context;
            use fig_ipc::get_fig_socket_path;

            if is_app_running() {
                return Ok(());
            }

            if opts.verbose {
                println!("\n→ Launching Fig...\n");
            }

            std::fs::remove_file(get_fig_socket_path()).ok();

            let process = std::process::Command::new("systemctl")
                .args(&["--user", "start", "fig"])
                .output()
                .context("\nUnable to launch Fig\n")?;

            if !process.status.success() {
                bail!("Failed to launch fig.desktop");
            }


            if !opts.wait_for_activation {
                return Ok(());
            }

            if !is_app_running() {
                anyhow::bail!("Unable to launch Fig");
            }

            // Wait for socket to exist
            let path = get_fig_socket_path();
            for _ in 0..9 {
                if path.exists() {
                    return Ok(());
                }
                // Sleep for a bit
                std::thread::sleep(std::time::Duration::from_millis(500));
            }

            bail!("\nUnable to finish launching Fig properly\n")
        } else {
            let _opts = opts;
            bail!("Fig desktop can not be launched on this platform")
        }
    }
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
        "\n{}\nLooks like you aren't logged in to fig, to login run: {}\n",
        "Not logged in".bold(),
        "fig login".magenta()
    )
}

pub fn get_fig_version() -> Result<(String, String)> {
    cfg_if! {
        if #[cfg(target_os = "macos")] {
            use anyhow::Context;
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
            let fig_build_number = get_plist_field("CFBundleVersion")?;
            Ok((fig_version, fig_build_number))
        } else {
            Err(anyhow::anyhow!("Unsupported platform"))
        }
    }
}

pub fn dialoguer_theme() -> impl dialoguer::theme::Theme {
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
