use anyhow::{Context, Result};
use regex::Regex;
use semver::Version;
use std::{
    env,
    fmt::Display,
    process::{exit, Command},
};

use dialoguer::theme::ColorfulTheme;

pub fn open_url(url: impl AsRef<str>) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(url.as_ref())
            .output()
            .with_context(|| "Could not open url")?;

        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(url.as_ref())
            .output()
            .with_context(|| "Could not open url")?;

        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .arg("/c")
            .arg("start")
            .arg(url.as_ref())
            .output()
            .with_context(|| "Could not open url")?;

        Ok(())
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    unimplemented!();
}

/// Ensure the command is being run with root privileges.
/// If not, rexecute the command with sudo.
pub fn permission_guard() -> Result<()> {
    #[cfg(unix)]
    {
        use nix::unistd::geteuid;

        // Hack to persist the ZDOTDIR environment variable to the new process.
        if let Some(val) = env::var_os("ZDOTDIR") {
            if env::var_os("FIG_ZDOTDIR").is_none() {
                env::set_var("FIG_ZDOTDIR", val);
            }
        }

        let sudo_prompt = match env::var("USER") {
            Ok(user) => format!("Please enter your password for user {}: ", user),
            Err(_) => "Please enter your password: ".into(),
        };

        match geteuid().is_root() {
            true => Ok(()),
            false => {
                let mut child = Command::new("sudo")
                    .arg("-E")
                    .arg("-p")
                    .arg(sudo_prompt)
                    .args(env::args_os())
                    .spawn()?;

                let status = child.wait()?;

                exit(status.code().unwrap_or(1));
            }
        }
    }

    #[cfg(windows)]
    {
        Ok(())
    }

    #[cfg(not(any(unix, windows)))]
    {
        Ok(())
    }
}

pub enum OSVersion {
    MacOS { version: Version, build: String },
    Linux { flavor: String },
    Windows { version: Version },
}

impl Display for OSVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OSVersion::MacOS { build, version } => {
                f.write_str(&format!("macOS {} {}", version, build))
            }
            OSVersion::Linux { flavor } => f.write_str(&format!("linux {}", flavor)),
            OSVersion::Windows { version } => f.write_str(&format!("windows {}", version)),
        }
    }
}

impl From<OSVersion> for String {
    fn from(os: OSVersion) -> Self {
        format!("{}", os)
    }
}

impl OSVersion {
    pub fn is_supported(&self) -> bool {
        match self {
            OSVersion::MacOS { version, build: _ } => version >= &Version::new(10, 14, 0),
            _ => false,
        }
    }
}

pub fn get_os_version() -> Result<OSVersion> {
    #[cfg(target_os = "macos")]
    {
        let version_info = Command::new("sw_vers")
            .output()
            .with_context(|| "Could not get macOS version")?;

        let version_info: String = String::from_utf8_lossy(&version_info.stdout).trim().into();

        let version_regex = Regex::new(r#"ProductVersion:\s*(\S+)"#).unwrap();
        let build_regex = Regex::new(r#"BuildVersion:\s*(\S+)"#).unwrap();

        let version = version_regex
            .captures(&version_info)
            .map(|c| c.get(1))
            .flatten()
            .map(|v| Version::parse(v.as_str()).ok())
            .flatten()
            .context("Invalid version")?;

        let build = build_regex
            .captures(&version_info)
            .map(|c| c.get(1))
            .flatten()
            .context("Invalid version")?
            .as_str();

        Ok(OSVersion::MacOS {
            version,
            build: build.into(),
        })
    }

    #[cfg(not(any(target_os = "macos")))]
    unimplemented!();
}

pub fn get_fig_version() -> Result<(String, String)> {
    #[cfg(target_os = "macos")]
    {
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
    }

    #[cfg(not(any(target_os = "macos")))]
    unimplemented!();
}

pub fn dialoguer_theme() -> impl dialoguer::theme::Theme {
    ColorfulTheme {
        prompt_prefix: dialoguer::console::style("?".into()).for_stderr().magenta(),
        ..ColorfulTheme::default()
    }
}
