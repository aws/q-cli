use std::process::Command;

/// The method of installation that Fig was installed with
pub enum InstallMethod {
    Brew,
    Unknown,
}

impl std::fmt::Display for InstallMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstallMethod::Brew => f.write_str("brew"),
            InstallMethod::Unknown => f.write_str("unknown"),
        }
    }
}

pub fn get_install_method() -> InstallMethod {
    if let Ok(output) = Command::new("brew").args(&["list", "fig", "-1"]).output() {
        if output.status.success() {
            return InstallMethod::Brew;
        }
    }

    InstallMethod::Unknown
}
