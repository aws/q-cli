use std::process::Command;

use once_cell::sync::Lazy;

static INSTALL_METHOD: Lazy<InstallMethod> = Lazy::new(|| {
    if let Ok(output) = Command::new("brew").args(&["list", "fig", "-1"]).output() {
        if output.status.success() {
            return InstallMethod::Brew;
        }
    }

    InstallMethod::Unknown
});

/// The method of installation that Fig was installed with
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallMethod {
    Brew,
    Unknown,
}

impl std::fmt::Display for InstallMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            InstallMethod::Brew => "brew",
            InstallMethod::Unknown => "unknown",
        })
    }
}

pub fn get_install_method() -> InstallMethod {
    *INSTALL_METHOD
}
