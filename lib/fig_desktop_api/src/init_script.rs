use std::collections::HashMap;
use std::env::{
    consts,
    var,
};

use camino::Utf8PathBuf;
use fig_util::directories;
#[cfg(target_os = "linux")]
use fig_util::system_info::linux::{
    get_desktop_environment,
    get_display_server,
    get_os_release,
    DesktopEnvironment,
    DisplayServer,
    OsRelease,
};
use serde::Serialize;
use serde_json::json;
use which::which;

const DEFAULT_THEMES: &[&str] = &["light", "dark", "system"];

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Serialize)]
struct LinuxConstants {
    display_server: Option<DisplayServer>,
    desktop_environment: Option<DesktopEnvironment>,
    os_release: Option<&'static OsRelease>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Constants {
    codewhisperer: bool,
    version: &'static str,
    cli: Option<Utf8PathBuf>,
    bundle_path: Option<Utf8PathBuf>,
    remote: Option<String>,
    home: Option<Utf8PathBuf>,
    fig_dot_dir: Option<Utf8PathBuf>,
    fig_data_dir: Option<Utf8PathBuf>,
    backups_dir: Option<Utf8PathBuf>,
    logs_dir: Option<Utf8PathBuf>,
    user: String,
    default_path: Option<String>,
    themes_folder: Option<Utf8PathBuf>,
    themes: Vec<String>,
    os: &'static str,
    arch: &'static str,
    env: HashMap<String, String>,
    new_uri_format: bool,
    #[cfg(target_os = "macos")]
    macos_version: String,
    #[cfg(target_os = "linux")]
    linux: LinuxConstants,
}

impl Default for Constants {
    fn default() -> Self {
        let themes_folder = directories::themes_dir()
            .ok()
            .and_then(|dir| Utf8PathBuf::try_from(dir).ok());

        let themes: Vec<String> = themes_folder
            .as_ref()
            .and_then(|path| {
                std::fs::read_dir(path).ok().map(|dir| {
                    dir.filter_map(|file| {
                        file.ok().and_then(|file| {
                            file.file_name()
                                .to_str()
                                .map(|name| name.strip_suffix(".json").unwrap_or(name))
                                .map(String::from)
                        })
                    })
                    .chain(DEFAULT_THEMES.iter().map(|s| (*s).to_owned()))
                    .collect()
                })
            })
            .unwrap_or_else(|| DEFAULT_THEMES.iter().map(|s| (*s).to_owned()).collect());

        Self {
            codewhisperer: true,
            version: env!("CARGO_PKG_VERSION"),
            cli: which("cw").ok().and_then(|exe| Utf8PathBuf::try_from(exe).ok()),
            bundle_path: None,
            remote: None,
            home: directories::home_dir_utf8().ok(),
            fig_dot_dir: directories::fig_data_dir_utf8().ok(),
            fig_data_dir: directories::fig_data_dir_utf8().ok(),
            backups_dir: directories::backups_dir_utf8().ok(),
            logs_dir: directories::logs_dir_utf8().ok(),
            user: whoami::username(),
            default_path: var("PATH").ok(),
            themes_folder,
            themes,
            os: consts::OS,
            arch: consts::ARCH,
            env: std::env::vars().collect(),
            new_uri_format: true,
            #[cfg(target_os = "macos")]
            macos_version: macos_utils::os::NSOperatingSystemVersion::get().to_string(),
            #[cfg(target_os = "linux")]
            linux: LinuxConstants {
                display_server: get_display_server().ok(),
                desktop_environment: get_desktop_environment().ok(),
                os_release: get_os_release(),
            },
        }
    }
}

impl Constants {
    pub fn script(&self) -> String {
        format!("fig.constants = {};", json!(self))
    }
}

pub fn javascript_init() -> String {
    [
        r#"if (!window.fig || !window.fig.quiet) console.log("[fig] declaring constants...");"#.into(),
        "if (!window.fig) window.fig = {};".into(),
        "if (!window.fig.constants) fig.constants = {};".into(),
        Constants::default().script(),
    ]
    .join("\n")
}
