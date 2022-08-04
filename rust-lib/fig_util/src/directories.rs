use std::ffi::OsStr;
use std::fmt::Display;
use std::path::{
    Path,
    PathBuf,
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DirectoryError {
    #[error("home directory not found")]
    NoHomeDirectory,
    #[error("non absolute path: {0:?}")]
    NonAbsolutePath(PathBuf),
}

use crate::Error as CrateError;

fn map_env_dir(path: &OsStr) -> Result<PathBuf, DirectoryError> {
    let path = Path::new(path);
    path.is_absolute()
        .then(|| path.to_path_buf())
        .ok_or_else(|| DirectoryError::NonAbsolutePath(path.to_owned()))
}

/// The $HOME directory
pub fn home_dir() -> Result<PathBuf, DirectoryError> {
    dirs::home_dir().ok_or(DirectoryError::NoHomeDirectory)
}

/// The $HOME/.fig directory
pub fn fig_dir() -> Result<PathBuf, DirectoryError> {
    match std::env::var_os("FIG_DOT_DIR") {
        Some(dot_dir) => map_env_dir(&dot_dir),
        None => dirs::home_dir()
            .ok_or(DirectoryError::NoHomeDirectory)
            .map(|p| p.join(".fig")),
    }
}

/// The $DATA/fig directory
pub fn fig_data_dir() -> Result<PathBuf, DirectoryError> {
    match std::env::var_os("FIG_DATA_DIR") {
        Some(data_dir) => map_env_dir(&data_dir),
        None => dirs::data_local_dir()
            .map(|path| path.join("fig"))
            .ok_or(DirectoryError::NoHomeDirectory),
    }
}

/// Get path to "/var/tmp/fig/$USERNAME/fig.socket"
pub fn fig_socket_path() -> Result<PathBuf, CrateError> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "linux")] {
            use std::path::Path;
            use std::process::Command;

            if wsl::is_wsl() {
                Ok(PathBuf::from(String::from_utf8_lossy(
                    &Command::new("wslpath").arg(String::from_utf8_lossy(
                        &Command::new("fig.exe").args(["_", "fig-socket-path"]
                    ).output()?.stdout).to_string()
                ).output()?.stdout).to_string()))
            } else {
                Ok([
                    Path::new("/var/tmp/fig"),
                    Path::new(&whoami::username()),
                    Path::new("fig.socket"),
                ]
                .into_iter()
                .collect())
            }
        } else if #[cfg(target_os = "macos")] {
            use std::path::Path;

            Ok([
                Path::new("/var/tmp/fig"),
                Path::new(&whoami::username()),
                Path::new("fig.socket"),
            ]
            .into_iter()
            .collect())
        } else if #[cfg(target_os = "windows")] {
            dirs::data_local_dir().map(|path| path.join("fig").join("fig.socket")).ok_or(CrateError::Directory(DirectoryError::NoHomeDirectory))
        } else {
            compile_error!("Unsupported platform");
        }
    }
}

pub fn figterm_socket_path(session_id: impl Display) -> Result<PathBuf, CrateError> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "linux")] {
            use std::process::Command;

            if wsl::is_wsl() {
                Ok(PathBuf::from(String::from_utf8_lossy(
                    &Command::new("wslpath").arg(String::from_utf8_lossy(
                        &Command::new("fig.exe").args(["_", "figterm-socket-path"]
                    ).output()?.stdout).to_string()
                ).output()?.stdout).to_string()))
            } else {
                Ok(PathBuf::from(format!("/tmp/figterm-{session_id}.socket")))
            }
        } else if #[cfg(target_os = "macos")] {
            Ok(PathBuf::from(format!("/tmp/figterm-{session_id}.socket")))
        } else if #[cfg(target_os = "windows")] {
            dirs::data_local_dir().map(|path| path.join("fig").join(format!("figterm-{session_id}.socket"))).ok_or(CrateError::Directory(DirectoryError::NoHomeDirectory))
        } else {
            compile_error!("Unsupported platform");
        }
    }
}

/// Get path to "$TMPDIR/fig/daemon.sock"
pub fn daemon_socket_path() -> PathBuf {
    [
        std::env::temp_dir().as_path(),
        Path::new("fig"),
        Path::new("daemon.sock"),
    ]
    .into_iter()
    .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn test() {
        assert_eq!(fig_dir().unwrap().file_name().unwrap(), ".fig");
        assert_eq!(fig_data_dir().unwrap().file_name().unwrap(), "fig");

        std::env::set_var("FIG_DOT_DIR", "/abc");
        std::env::set_var("FIG_DATA_DIR", "/def");

        assert_eq!(fig_dir().unwrap().file_name().unwrap(), "abc");
        assert_eq!(fig_data_dir().unwrap().file_name().unwrap(), "def");

        std::env::set_var("FIG_DOT_DIR", "abc");
        std::env::set_var("FIG_DATA_DIR", "def");

        fig_dir().unwrap_err();
        fig_data_dir().unwrap_err();
    }

    #[cfg(windows)]
    #[test]
    fn test() {
        assert_eq!(fig_dir().unwrap().file_name().unwrap(), ".fig");
        assert_eq!(fig_data_dir().unwrap().file_name().unwrap(), "fig");

        std::env::set_var("FIG_DOT_DIR", "c:\\abc");
        std::env::set_var("FIG_DATA_DIR", "c:\\def");

        assert_eq!(fig_dir().unwrap().file_name().unwrap(), "abc");
        assert_eq!(fig_data_dir().unwrap().file_name().unwrap(), "def");

        std::env::set_var("FIG_DOT_DIR", "abc");
        std::env::set_var("FIG_DATA_DIR", "def");

        fig_dir().unwrap_err();
        fig_data_dir().unwrap_err();
    }
}
