pub mod linux;

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

pub fn in_ssh() -> bool {
    *IN_SSH
}

/// Test if the program is running under WSL
pub fn in_wsl() -> bool {
    cfg_if! {
        if #[cfg(target_os = "macos")] {
            false
        } else if #[cfg(target_os = "linux")] {
            *IN_WSL
        } else if #[cfg(target_os = "windows")] {
            false
        }
    }
}

/// Is Fig running on a remote instance
pub fn is_remote() -> bool {
    // TODO(chay): Add detection for inside docker container
    in_ssh() || in_wsl()
}

pub fn get_system_id() -> Result<String, Error> {
    #[allow(unused_assignments)]
    let mut hwid = None;

    cfg_if!(
        if #[cfg(target_os = "macos")] {
            let output = std::process::Command::new("ioreg")
                .args(&["-rd1", "-c", "IOPlatformExpertDevice"])
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

            hwid = Some(machine_id);
        } else if #[cfg(target_os = "linux")] {
            for path in ["/var/lib/dbus/machine-id", "/etc/machine-id"] {
                use std::io::Read;

                if std::path::Path::new(path).exists() {
                    let content = {
                        let mut file = std::fs::File::open(path)?;
                        let mut content = String::new();
                        file.read_to_string(&mut content)?;
                        content
                    };
                    hwid = Some(content);
                    break;
                }
            }
        } else if #[cfg(windows)] {
            use winreg::enums::HKEY_LOCAL_MACHINE;
            use winreg::RegKey;

            let rkey = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(r"SOFTWARE\Microsoft\Cryptography")?;
            let id: String = rkey.get_value("MachineGuid")?;

            hwid = Some(id);
        }
    );

    let mut hasher = Sha256::new();
    hasher.update(hwid.ok_or(Error::HwidNotFound)?);
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
        os_release: Option<linux::OsRelease>,
    },
    Windows {
        version: String,
        build: Option<String>,
    },
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
            OSVersion::Windows { version, .. } => write!(f, "Windows {version}"),
        }
    }
}

impl OSVersion {
    pub fn new() -> Option<OSVersion> {
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
                use std::process::Command;

                let output = Command::new("systeminfo").arg("/FO").arg("CSV").output().ok()?;

                Some(OSVersion::Windows {
                    version: String::from_utf8_lossy(&output.stdout)
                        .split_once('\n')
                        .unwrap()
                        .1
                        .split(',')
                        .nth(2)
                        .unwrap()
                        .trim_matches('"')
                        .to_owned(),
                    build: None
                })
            } else {
                compile_error!("Unsupported platform")
            }
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
            },
            OSVersion::Linux { .. } => SupportLevel::InDevelopment,
            OSVersion::Windows { .. } => SupportLevel::InDevelopment,
        }
    }
}
