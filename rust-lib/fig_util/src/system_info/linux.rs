use once_cell::sync::Lazy;
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
        // x11 is not guarentee this var is set, so we just assume x11 if it is not set
        _ => Ok(DisplayServer::X11),
    }
}

pub fn get_desktop_environment() -> Result<DesktopEnvironment, Error> {
    match std::env::var("XDG_CURRENT_DESKTOP") {
        Ok(current) => {
            let current = current.to_lowercase();
            let (_, desktop) = current.split_once(':').unwrap_or(("", current.as_str()));
            match desktop.to_lowercase().as_str() {
                "gnome" | "gnome-xorg" | "ubuntu" => Ok(DesktopEnvironment::Gnome),
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

/// https://www.man7.org/linux/man-pages/man5/os-release.5.html
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

#[cfg(all(test, target_os = "linux"))]
mod test {
    use super::*;

    #[test]
    fn os_release() {
        OsRelease::new().unwrap();
    }
}
