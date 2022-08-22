use cfg_if::cfg_if;
use sha2::{
    Digest,
    Sha256,
};

use crate::Error;

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

            let rkey = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey("SOFTWARE\\Microsoft\\Cryptography")?;
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

#[cfg(target_os = "linux")]
mod linux {
    use once_cell::sync::Lazy;
    use serde::{
        Deserialize,
        Serialize,
    };

    use super::*;

    #[derive(Debug)]
    pub enum DisplayServer {
        X11,
        Wayland,
    }

    #[derive(Debug)]
    pub enum DesktopEnvironment {
        Gnome,
        Plasma,
        I3,
    }

    pub fn detect_desktop() -> Result<(DisplayServer, DesktopEnvironment), Error> {
        let display_server = match std::env::var("XDG_SESSION_TYPE") {
            Ok(session) => match session.as_str() {
                "x11" => DisplayServer::X11,
                "wayland" => DisplayServer::Wayland,
                _ => return Err(Error::UnknownDisplayServer(session)),
            },
            _ => return Err(Error::MissingEnv("XDG_SESSION_TYPE")),
        };

        let desktop_environment = match std::env::var("XDG_SESSION_DESKTOP") {
            Ok(desktop) => match desktop.as_str() {
                "GNOME" => DesktopEnvironment::Gnome,
                "KDE" => DesktopEnvironment::Plasma,
                "i3" => DesktopEnvironment::I3,
                _ => return Err(Error::UnknownDesktop(desktop)),
            },
            _ => return Err(Error::MissingEnv("XDG_SESSION_DESKTOP")),
        };

        Ok((display_server, desktop_environment))
    }

    static LINUX_OS_RELEASE: Lazy<Option<LinuxOsRelease>> = Lazy::new(LinuxOsRelease::new);

    pub fn get_linux_os_release() -> Option<&'static LinuxOsRelease> {
        LINUX_OS_RELEASE.as_ref()
    }

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct LinuxOsRelease {
        pub name: Option<String>,
        pub version: Option<String>,
        pub id: Option<String>,
        pub version_id: Option<String>,
        pub pretty_name: Option<String>,
        pub cpe_name: Option<String>,
        pub build_id: Option<String>,
        pub variant: Option<String>,
        pub variant_id: Option<String>,
    }

    impl LinuxOsRelease {
        pub(crate) fn new() -> Option<LinuxOsRelease> {
            match std::fs::read_to_string("/etc/os-release") {
                Ok(release) => {
                    let mut os_release = LinuxOsRelease::default();
                    for line in release.lines() {
                        if let Some((key, value)) = line.split_once('=') {
                            match key {
                                "NAME" => os_release.name = Some(value.into()),
                                "VERSION" => os_release.version = Some(value.into()),
                                "ID" => os_release.id = Some(value.into()),
                                "VERSION_ID" => os_release.version_id = Some(value.into()),
                                "PRETTY_NAME" => os_release.pretty_name = Some(value.into()),
                                "CPE_NAME" => os_release.cpe_name = Some(value.into()),
                                "BUILD_ID" => os_release.build_id = Some(value.into()),
                                "VARIANT" => os_release.variant = Some(value.into()),
                                "VARIANT_ID" => os_release.variant_id = Some(value.into()),
                                _ => {},
                            }
                        }
                    }
                    Some(os_release)
                },
                Err(_) => None,
            }
        }
    }
}

#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(test)]
mod test {
    use super::*;

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_os_release() {
        LinuxOsRelease::new().unwrap();
    }
}
