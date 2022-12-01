pub mod backoff;
pub mod spinner;
pub mod sync;

use std::env;
use std::ffi::OsStr;
use std::io::stdout;
use std::iter::empty;
use std::path::{
    Path,
    PathBuf,
};
use std::process::Command;
use std::time::Duration;

use cfg_if::cfg_if;
use crossterm::style::Stylize;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{
    FuzzySelect,
    Select,
};
use eyre::{
    bail,
    ContextCompat,
    Result,
};
use fig_ipc::local::quit_command;
use fig_util::consts::FIG_BUNDLE_ID;
use fig_util::is_fig_desktop_running;
use globset::{
    Glob,
    GlobSet,
    GlobSetBuilder,
};
use once_cell::sync::Lazy;
use regex::Regex;
use tracing::warn;

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

pub async fn quit_fig(verbose: bool) -> Result<()> {
    if !is_fig_desktop_running() {
        if verbose {
            println!("Fig is not running");
        }
        return Ok(());
    }

    let telem_join = match verbose {
        true => {
            println!("Quitting Fig");

            Some(tokio::spawn(async {
                fig_telemetry::dispatch_emit_track(
                    fig_telemetry::TrackEvent::new(
                        fig_telemetry::TrackEventType::QuitApp,
                        fig_telemetry::TrackSource::Cli,
                        env!("CARGO_PKG_VERSION").into(),
                        empty::<(&str, &str)>(),
                    ),
                    false,
                    true,
                )
                .await
                .ok();
            }))
        },
        false => None,
    };

    if quit_command().await.is_err() {
        tokio::time::sleep(Duration::from_millis(500)).await;
        let second_try = quit_command().await;
        if second_try.is_err() {
            cfg_if! {
                if #[cfg(target_os = "linux")] {
                    if let Ok(output) = Command::new("killall").arg("fig_desktop").output() {
                        if output.status.success() {
                            return Ok(());
                        }
                    }
                } else if #[cfg(target_os = "macos")] {
                    if let Ok(info) = get_app_info() {
                        let pid = Regex::new(r"pid = (\S+)")
                            .unwrap()
                            .captures(&info)
                            .and_then(|c| c.get(1));
                        if let Some(pid) = pid {
                            let success = Command::new("kill")
                                .arg("-KILL")
                                .arg(pid.as_str())
                                .status()
                                .map(|res| res.success());
                            if let Ok(true) = success {
                                return Ok(());
                            }
                        }
                    }
                } else if #[cfg(target_os = "windows")] {
                    // TODO(chay): Add windows behavior here
                }
            }
            if verbose {
                println!("Unable to quit Fig");
            }

            second_try?;
        }
    }

    telem_join.map(|f| async { f.await.ok() });

    Ok(())
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

pub fn choose_fuzzy(prompt: &str, options: &[impl ToString]) -> Result<usize> {
    tokio::spawn(async {
        tokio::signal::ctrl_c().await.unwrap();
        crossterm::execute!(stdout(), crossterm::cursor::Show).unwrap();
        std::process::exit(0);
    });

    if options.is_empty() {
        bail!("no options passed to choose")
    }

    if !*IS_TTY {
        warn!("choose called without TTY, choosing first option");
        return Ok(0);
    }

    FuzzySelect::with_theme(&dialoguer_theme())
        .items(options)
        .default(0)
        .with_prompt(prompt)
        .interact_opt()?
        .ok_or_else(|| eyre::eyre!("Cancelled"))
}

pub fn choose(prompt: &str, options: &[impl ToString]) -> Result<usize> {
    tokio::spawn(async {
        tokio::signal::ctrl_c().await.unwrap();
        crossterm::execute!(stdout(), crossterm::cursor::Show).unwrap();
        std::process::exit(0);
    });

    if options.is_empty() {
        bail!("no options passed to choose")
    }

    if !*IS_TTY {
        warn!("choose called without TTY, choosing first option");
        return Ok(0);
    }

    Select::with_theme(&dialoguer_theme())
        .items(options)
        .default(0)
        .with_prompt(prompt)
        .interact_opt()?
        .ok_or_else(|| eyre::eyre!("Cancelled"))
}

pub fn get_running_app_info(bundle_id: impl AsRef<str>, field: impl AsRef<str>) -> Result<String> {
    let info = Command::new("lsappinfo")
        .args(["info", "-only", field.as_ref(), "-app", bundle_id.as_ref()])
        .output()?;
    let info = String::from_utf8(info.stdout)?;
    let value = info
        .split('=')
        .nth(1)
        .context(eyre::eyre!("Could not get field value for {}", field.as_ref()))?
        .replace('"', "");
    Ok(value.trim().into())
}

pub fn get_app_info() -> Result<String> {
    let output = Command::new("lsappinfo")
        .args(["info", "-app", FIG_BUNDLE_ID])
        .output()?;
    let result = String::from_utf8(output.stdout)?;
    Ok(result.trim().into())
}

pub fn dialoguer_theme() -> ColorfulTheme {
    ColorfulTheme {
        prompt_prefix: dialoguer::console::style("?".into()).for_stderr().magenta(),
        ..ColorfulTheme::default()
    }
}

#[cfg(target_os = "macos")]
pub async fn is_brew_reinstall() -> bool {
    use bstr::ByteSlice;

    tokio::process::Command::new("ps")
        .args(["aux", "-o", "args"])
        .output()
        .await
        .map_or(false, |output| {
            output.stdout.contains_str(b"brew upgrade") || output.stdout.contains_str(b"brew reinstall")
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regex() {
        let regex_test = |regex: &str, input: &str, expected: Option<&str>| {
            assert_eq!(match_regex(regex, input), expected.map(|s| s.into()));
        };

        regex_test(r"foo=(\S+)", "foo=bar", Some("bar"));
        regex_test(r"foo=(\S+)", "bar=foo", None);
        regex_test(r"foo=(\S+)", "foo=bar baz", Some("bar"));
        regex_test(r"foo=(\S+)", "foo=", None);
    }

    #[test]
    fn exe_path() {
        #[cfg(unix)]
        assert!(is_executable_in_path("cargo"));

        #[cfg(windows)]
        assert!(is_executable_in_path("cargo.exe"));
    }

    #[test]
    fn globs() {
        let set = glob(["*.txt", "*.md"]).unwrap();
        assert!(set.is_match("README.md"));
        assert!(set.is_match("LICENSE.txt"));
    }

    #[ignore]
    #[test]
    fn sysinfo_test() {
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
}
