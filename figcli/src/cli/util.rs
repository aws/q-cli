use anyhow::{Context, Result};
use crossterm::style::Stylize;
use dialoguer::theme::ColorfulTheme;
use serde::{Deserialize, Serialize};
use std::{
    env,
    fmt::Display,
    process::{exit, Command},
};

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

/// The support level for different platforms
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupportLevel {
    /// A fully supported platform
    Supported,
    /// A platform that is currently in development
    InDevelopment,
    /// A platform that is not supported
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OSVersion {
    MacOS {
        major: i32,
        minor: i32,
        patch: Option<i32>,
        build: String,
    },
    Linux {
        kernel_version: String,
        distribution: Option<String>,
        release: Option<String>,
    },
    Windows {
        version: String,
    },
}

impl Display for OSVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OSVersion::MacOS {
                major,
                minor,
                patch,
                build,
            } => {
                let patch = patch.unwrap_or(0);
                f.write_str(&format!("macOS {major}.{minor}.{patch} ({build})",))
            }
            OSVersion::Linux { kernel_version, .. } => {
                f.write_str(&format!("Linux {kernel_version}"))
            }
            OSVersion::Windows { version } => f.write_str(&format!("Windows {version}")),
        }
    }
}

impl From<OSVersion> for String {
    fn from(os: OSVersion) -> Self {
        format!("{os}")
    }
}

impl OSVersion {
    pub fn new() -> Result<OSVersion> {
        #[cfg(target_os = "macos")]
        {
            use regex::Regex;

            let version_info = Command::new("sw_vers")
                .output()
                .with_context(|| "Could not get macOS version")?;

            let version_info: String = String::from_utf8_lossy(&version_info.stdout).trim().into();

            let version_regex = Regex::new(r#"ProductVersion:\s*(\S+)"#).unwrap();
            let build_regex = Regex::new(r#"BuildVersion:\s*(\S+)"#).unwrap();

            let version: String = version_regex
                .captures(&version_info)
                .and_then(|c| c.get(1))
                .map(|v| v.as_str().into())
                .context("Invalid version")?;

            let major = version
                .split('.')
                .next()
                .context("Invalid version")?
                .parse()?;

            let minor = version
                .split('.')
                .nth(1)
                .context("Invalid version")?
                .parse()?;

            let patch = version.split('.').nth(2).and_then(|p| p.parse().ok());

            let build = build_regex
                .captures(&version_info)
                .and_then(|c| c.get(1))
                .context("Invalid version")?
                .as_str()
                .into();

            Ok(OSVersion::MacOS {
                major,
                minor,
                patch,
                build,
            })
        }

        #[cfg(target_os = "linux")]
        {
            use nix::sys::utsname::uname;
            // use regex::Regex;

            let uname = uname();
            let kernel_version = uname.release().to_owned();

            // let version_info = Command::new("lsb_release")
            //     .arg("-a")
            //     .output()
            //     .with_context(|| "Could not get Linux version")?;

            // let version_info: String = String::from_utf8_lossy(&version_info.stdout).trim().into();

            // let distribution_regex = Regex::new(r#"Distributor ID:\s*(\S+)"#).unwrap();
            // let kernel_regex = Regex::new(r#"Description:\s*(\S+)"#).unwrap();

            // let flavor = distribution_regex
            //     .captures(&version_info)
            //     .and_then(|c| c.get(1))
            //     .map(|v| v.as_str().into())
            //     .context("Invalid version")?;

            // let kernel_version = kernel_regex
            //     .captures(&version_info)
            //     .and_then(|c| c.get(1))
            //     .map(|v| v.as_str().into())
            //     .context("Invalid version")?;

            Ok(OSVersion::Linux {
                kernel_version,
                distribution: None,
                release: None,
            })
        }
    }

    pub fn support_level(&self) -> SupportLevel {
        match self {
            OSVersion::MacOS { major, minor, .. } => {
                // Minimum supported macOS version is 10.14.0
                if *major > 10 || (*major == 10 && *minor >= 14) {
                    SupportLevel::Supported
                } else {
                    SupportLevel::Unsupported
                }
            }
            OSVersion::Linux { .. } => SupportLevel::InDevelopment,
            _ => SupportLevel::Unsupported,
        }
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
    #[cfg(target_os = "macos")]
    {
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
