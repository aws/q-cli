pub mod linux;

use std::borrow::Cow;

use cfg_if::cfg_if;
use once_cell::sync::Lazy;
use serde::{
    Deserialize,
    Serialize,
};
use sha2::{
    Digest,
    Sha256,
};

use crate::Error;

static OS_VERSION: Lazy<Option<OSVersion>> = Lazy::new(|| {
    cfg_if! {
        if #[cfg(target_os = "macos")] {
            use std::process::Command;
            use regex::Regex;

            let version_info = Command::new("sw_vers")
                .output()
                .ok()?;

            let version_info: String = String::from_utf8_lossy(&version_info.stdout).trim().into();

            let version_regex = Regex::new(r#"ProductVersion:\s*(\S+)"#).unwrap();
            let build_regex = Regex::new(r#"BuildVersion:\s*(\S+)"#).unwrap();

            let version: String = version_regex
                .captures(&version_info)
                .and_then(|c| c.get(1))
                .map(|v| v.as_str().into())?;

            let major = version
                .split('.')
                .next()?
                .parse().ok()?;

            let minor = version
                .split('.')
                .nth(1)?
                .parse().ok()?;

            let patch = version.split('.').nth(2).and_then(|p| p.parse().ok());

            let build = build_regex
                .captures(&version_info)
                .and_then(|c| c.get(1))?
                .as_str()
                .into();

            Some(OSVersion::MacOS {
                major,
                minor,
                patch,
                build,
            })
        } else if #[cfg(target_os = "linux")] {
            use nix::sys::utsname::uname;

            let kernel_version = uname().ok()?.release().to_string_lossy().into();
            let os_release = linux::get_os_release().cloned();

            Some(OSVersion::Linux {
                kernel_version,
                os_release,
            })
        } else if #[cfg(target_os = "windows")] {
            use winreg::enums::HKEY_LOCAL_MACHINE;
            use winreg::RegKey;

            let rkey = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion").ok()?;
            let build: String = rkey.get_value("CurrentBuild").ok()?;

            Some(OSVersion::Windows {
                name: rkey.get_value("ProductName").ok()?,
                build: build.parse::<u32>().ok()?,
            })
        } else if #[cfg(target_os = "freebsd")] {
            use nix::sys::utsname::uname;

            let version = uname().ok()?.release().to_string_lossy().into();

            Some(OSVersion::FreeBsd {
                version,
            })

        }
    }
});

#[cfg(target_os = "linux")]
static IN_WSL: Lazy<bool> = Lazy::new(|| {
    if let Ok(b) = std::fs::read("/proc/sys/kernel/osrelease") {
        if let Ok(s) = std::str::from_utf8(&b) {
            let a = s.to_ascii_lowercase();
            return a.contains("microsoft") || a.contains("wsl");
        }
    }
    false
});

static IN_SSH: Lazy<bool> = Lazy::new(|| {
    std::env::var_os("SSH_CLIENT").is_some()
        || std::env::var_os("SSH_CONNECTION").is_some()
        || std::env::var_os("SSH_TTY").is_some()
});

static HAS_PARENT: Lazy<bool> = Lazy::new(|| std::env::var_os("FIG_PARENT").is_some());

static IN_CODESPACES: Lazy<bool> =
    Lazy::new(|| std::env::var_os("CODESPACES").is_some() || std::env::var_os("FIG_CODESPACES").is_some());

static IN_CI: Lazy<bool> = Lazy::new(|| std::env::var_os("CI").is_some() || std::env::var_os("FIG_CI").is_some());

/// The support level for different platforms
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupportLevel {
    /// A fully supported platform
    Supported,
    /// Supported, but with a caveat
    SupportedWithCaveat { info: Cow<'static, str> },
    /// A platform that is currently in development
    InDevelopment { info: Option<Cow<'static, str>> },
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
        os_release: Option<linux::OsRelease>,
    },
    Windows {
        name: String,
        build: u32,
    },
    FreeBsd {
        version: String,
    },
}

impl OSVersion {
    pub fn support_level(&self) -> SupportLevel {
        match self {
            OSVersion::MacOS { major, minor, .. } => {
                // Minimum supported macOS version is 10.14.0
                if *major > 10 || (*major == 10 && *minor >= 14) {
                    SupportLevel::Supported
                } else {
                    SupportLevel::Unsupported
                }
            },
            OSVersion::Linux { .. } => {
                if crate::manifest::is_full() {
                    SupportLevel::InDevelopment {
                        info: Some(
                            "Autocomplete is currently in alpha for Linux, other products should work as expected."
                                .into(),
                        ),
                    }
                } else {
                    SupportLevel::SupportedWithCaveat {
                        info: "Autocomplete is not yet available on Linux, but other products should work as expected."
                            .into(),
                    }
                }
            },
            OSVersion::Windows { build, .. } => match build {
                // Only Windows 11 is fully supported at the moment
                build if *build >= 22000 => SupportLevel::Supported,
                // Windows 10 development has known issues
                build if *build >= 10240 => SupportLevel::InDevelopment {
                    info: Some(
                        "Since support for Windows 10 is still in progress,\
Autocomplete only works in Git Bash with the default prompt.\
Please upgrade to Windows 11 or wait for a fix while we work this issue out."
                            .into(),
                    ),
                },
                // Earlier versions of Windows are not supported
                _ => SupportLevel::Unsupported,
            },
            OSVersion::FreeBsd { .. } => SupportLevel::InDevelopment { info: None },
        }
    }

    pub fn user_readable(&self) -> Vec<String> {
        match self {
            OSVersion::Linux {
                kernel_version,
                os_release,
            } => {
                let mut v = vec![format!("kernel: {kernel_version}")];

                if let Some(os_release) = os_release {
                    if let Some(name) = &os_release.name {
                        v.push(format!("distro: {name}"));
                    }

                    if let Some(version) = &os_release.version {
                        v.push(format!("distro-version: {version}"));
                    } else if let Some(version) = &os_release.version_id {
                        v.push(format!("distro-version: {version}"));
                    }

                    if let Some(variant) = &os_release.variant {
                        v.push(format!("distro-variant: {variant}"));
                    } else if let Some(variant) = &os_release.variant_id {
                        v.push(format!("distro-variant: {variant}"));
                    }

                    if let Some(build) = &os_release.build_id {
                        v.push(format!("distro-build: {build}"));
                    }
                }

                v
            },
            other => vec![format!("{other}")],
        }
    }
}

impl std::fmt::Display for OSVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OSVersion::MacOS {
                major,
                minor,
                patch,
                build,
            } => {
                let patch = patch.unwrap_or(0);
                write!(f, "macOS {major}.{minor}.{patch} ({build})")
            },
            OSVersion::Linux {
                kernel_version,
                os_release,
            } => match os_release
                .as_ref()
                .and_then(|r| r.pretty_name.as_ref().or(r.name.as_ref()))
            {
                Some(distro_name) => write!(f, "Linux {kernel_version} - {distro_name}"),
                None => write!(f, "Linux {kernel_version}"),
            },
            OSVersion::Windows { name, build } => write!(f, "{name} (or newer) - build {build}"),
            OSVersion::FreeBsd { version } => write!(f, "FreeBSD {version}"),
        }
    }
}

pub fn os_version() -> Option<&'static OSVersion> {
    OS_VERSION.as_ref()
}

pub fn in_ssh() -> bool {
    *IN_SSH
}

/// Test if the program is running under WSL
pub fn in_wsl() -> bool {
    cfg_if! {
        if #[cfg(target_os = "linux")] {
            *IN_WSL
        } else {
            false
        }
    }
}

/// Is Fig running on a remote instance
pub fn is_remote() -> bool {
    // TODO(chay): Add detection for inside docker container
    in_ssh() || in_wsl() || std::env::var_os("FIG_FAKE_IS_REMOTE").is_some()
}

/// Whether Fig has a parent. Determines if we have an IPC path to a Desktop app from a remote
/// environment
pub fn has_parent() -> bool {
    *HAS_PARENT
}

pub fn in_codespaces() -> bool {
    *IN_CODESPACES
}

pub fn in_ci() -> bool {
    *IN_CI
}

#[cfg(target_os = "macos")]
fn raw_system_id() -> Result<String, Error> {
    let output = std::process::Command::new("ioreg")
        .args(["-rd1", "-c", "IOPlatformExpertDevice"])
        .output()?;

    let output = String::from_utf8_lossy(&output.stdout);

    let machine_id: String = output
        .lines()
        .find(|line| line.contains("IOPlatformUUID"))
        .ok_or(Error::HwidNotFound)?
        .split('=')
        .nth(1)
        .ok_or(Error::HwidNotFound)?
        .trim()
        .trim_start_matches('"')
        .trim_end_matches('"')
        .into();

    Ok(machine_id)
}

#[cfg(target_os = "linux")]
fn raw_system_id() -> Result<String, Error> {
    for path in ["/var/lib/dbus/machine-id", "/etc/machine-id"] {
        use std::io::Read;

        if std::path::Path::new(path).exists() {
            let content = {
                let mut file = std::fs::File::open(path)?;
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                content
            };
            return Ok(content);
        }
    }
    Err(Error::HwidNotFound)
}

#[cfg(target_os = "windows")]
fn raw_system_id() -> Result<String, Error> {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;

    let rkey = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(r"SOFTWARE\Microsoft\Cryptography")?;
    let id: String = rkey.get_value("MachineGuid")?;

    Ok(id)
}

#[cfg(target_os = "freebsd")]
fn raw_system_id() -> Result<String, Error> {
    Err(Error::HwidNotFound)
}

pub fn get_system_id() -> Result<String, Error> {
    let hwid = raw_system_id()?;
    let mut hasher = Sha256::new();
    hasher.update(hwid);
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn get_platform() -> &'static str {
    if let Some(over_ride) = option_env!("FIG_OVERRIDE_PLATFORM") {
        over_ride
    } else {
        std::env::consts::OS
    }
}

pub fn get_arch() -> &'static str {
    if let Some(over_ride) = option_env!("FIG_OVERRIDE_ARCH") {
        over_ride
    } else {
        std::env::consts::ARCH
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_system_id() {
        let id = get_system_id();
        assert!(id.is_ok());
        assert_eq!(id.unwrap().len(), 64);
    }
}
