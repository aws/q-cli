use std::path::Path;

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{
    Deserialize,
    Serialize,
};

use crate::Error;

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

pub fn get_display_server() -> Result<DisplayServer, Error> {
    match std::env::var("XDG_SESSION_TYPE") {
        Ok(session) => match session.as_str() {
            "x11" => Ok(DisplayServer::X11),
            "wayland" => Ok(DisplayServer::Wayland),
            _ => Err(Error::UnknownDisplayServer(session)),
        },
        // x11 is not guarantee this var is set, so we just assume x11 if it is not set
        _ => Ok(DisplayServer::X11),
    }
}

pub fn get_desktop_environment() -> Result<DesktopEnvironment, Error> {
    match std::env::var("XDG_CURRENT_DESKTOP") {
        Ok(current) => {
            let current = current.to_lowercase();
            let (_, desktop) = current.split_once(':').unwrap_or(("", current.as_str()));
            match desktop.to_lowercase().as_str() {
                "gnome" | "gnome-xorg" | "ubuntu" | "pop" => Ok(DesktopEnvironment::Gnome),
                "kde" | "plasma" => Ok(DesktopEnvironment::Plasma),
                "i3" => Ok(DesktopEnvironment::I3),
                _ => Err(Error::UnknownDesktop(current)),
            }
        },
        _ => Err(Error::MissingEnv("XDG_CURRENT_DESKTOP")),
    }
}

static OS_RELEASE: Lazy<Option<OsRelease>> = Lazy::new(OsRelease::new);

pub fn get_os_release() -> Option<&'static OsRelease> {
    OS_RELEASE.as_ref()
}

/// Fields from <https://www.man7.org/linux/man-pages/man5/os-release.5.html>
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OsRelease {
    pub id: Option<String>,

    pub name: Option<String>,
    pub pretty_name: Option<String>,

    pub version_id: Option<String>,
    pub version: Option<String>,

    pub build_id: Option<String>,

    pub variant_id: Option<String>,
    pub variant: Option<String>,
}

impl OsRelease {
    pub(crate) fn new() -> Option<OsRelease> {
        match std::fs::read_to_string("/etc/os-release") {
            Ok(release) => {
                let mut os_release = OsRelease::default();
                for line in release.lines() {
                    if let Some((key, value)) = line.split_once('=') {
                        match key {
                            "ID" => os_release.id = Some(value.into()),

                            "NAME" => os_release.name = Some(value.into()),
                            "PRETTY_NAME" => os_release.name = Some(value.into()),

                            "VERSION" => os_release.version = Some(value.into()),
                            "VERSION_ID" => os_release.version_id = Some(value.into()),

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

static CONTAINERENV_ENGINE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"engine="([^"\s]+)""#).unwrap());

pub enum SandboxKind {
    None,
    Flatpak,
    Snap,
    Docker,
    Container(Option<String>),
}

pub fn detect_sandbox() -> SandboxKind {
    if Path::new("/.flatpak-info").exists() {
        return SandboxKind::Flatpak;
    }
    if std::env::var("SNAP").is_ok() {
        return SandboxKind::Snap;
    }
    if Path::new("/.dockerenv").exists() {
        return SandboxKind::Docker;
    }
    if let Ok(env) = std::fs::read_to_string("/var/run/.containerenv") {
        return SandboxKind::Container(
            CONTAINERENV_ENGINE
                .captures(&env)
                .and_then(|x| x.get(1))
                .map(|x| x.as_str().to_string()),
        );
    }

    SandboxKind::None
}

impl SandboxKind {
    pub fn is_container(&self) -> bool {
        matches!(self, SandboxKind::Docker | SandboxKind::Container(_))
    }

    pub fn is_app_runtime(&self) -> bool {
        matches!(self, SandboxKind::Flatpak | SandboxKind::Snap)
    }

    pub fn is_none(&self) -> bool {
        matches!(self, SandboxKind::None)
    }
}

#[cfg(all(test, target_os = "linux"))]
mod test {
    use super::*;

    #[test]
    fn os_release() {
        OsRelease::new().unwrap();
    }
}
