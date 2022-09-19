use std::convert::TryInto;
use std::env;
use std::fmt::Display;
use std::path::PathBuf;

use camino::Utf8PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DirectoryError {
    #[error("home directory not found")]
    NoHomeDirectory,
    #[error("non absolute path: {0:?}")]
    NonAbsolutePath(PathBuf),
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Utf8FromPath(#[from] camino::FromPathError),
    #[error(transparent)]
    Utf8FromPathBuf(#[from] camino::FromPathBufError),
}

type Result<T, E = DirectoryError> = std::result::Result<T, E>;

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn map_env_dir(path: &std::ffi::OsStr) -> Result<PathBuf> {
    let path = std::path::Path::new(path);
    path.is_absolute()
        .then(|| path.to_path_buf())
        .ok_or_else(|| DirectoryError::NonAbsolutePath(path.to_owned()))
}

/// The $HOME directory
pub fn home_dir() -> Result<PathBuf> {
    dirs::home_dir().ok_or(DirectoryError::NoHomeDirectory)
}

/// The $HOME/.fig directory
pub fn fig_dir() -> Result<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(any(target_os = "linux", target_os = "macos"))] {
            match std::env::var_os("FIG_DOT_DIR") {
                Some(dot_dir) => map_env_dir(&dot_dir),
                None => dirs::home_dir()
                    .ok_or(DirectoryError::NoHomeDirectory)
                    .map(|p| p.join(".fig")),
            }
        } else if #[cfg(target_os = "windows")] {
            Ok(dirs::data_local_dir().ok_or(DirectoryError::NoHomeDirectory)?.join("Fig"))
        }
    }
}

/// The fig data directory
///
/// - Linux: `$XDG_DATA_HOME/fig or $HOME/.local/share/fig`
/// - MacOS: `$HOME/Library/Application Support/fig`
/// - Windows: `%APPDATA%/fig`
pub fn fig_data_dir() -> Result<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(any(target_os = "linux", target_os = "macos"))] {
            match std::env::var_os("FIG_DATA_DIR") {
                Some(data_dir) => map_env_dir(&data_dir),
                None => dirs::data_local_dir()
                    .map(|path| path.join("fig"))
                    .ok_or(DirectoryError::NoHomeDirectory),
            }
        } else if #[cfg(target_os = "windows")] {
            Ok(fig_dir()?.join("userdata"))
        }
    }
}

/// The ephemeral fig state directory
///
/// - Linux/MacOS: `/var/tmp/fig/$USER`
/// - Windows: ???
pub fn fig_ephemeral_dir() -> Result<PathBuf> {
    named_fig_ephemeral_dir(whoami::username())
}

pub fn named_fig_ephemeral_dir(name: String) -> Result<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "linux")] {
            use std::path::Path;
            use std::process::Command;

            if crate::system_info::in_wsl() {
                let socket_path = PathBuf::from(String::from_utf8_lossy(
                    &Command::new("wslpath").arg(String::from_utf8_lossy(
                        &Command::new("fig.exe").args(["_", "fig-socket-path"]
                    ).output()?.stdout).to_string()
                ).output()?.stdout).to_string());
                let dir_path = socket_path.parent()
                    .and_then(|p| p.parent()).ok_or(DirectoryError::NoHomeDirectory)?;
                Ok(dir_path.join(name))
            } else {
                Ok(Path::new("/var/tmp/fig").join(name))
            }
        } else if #[cfg(target_os = "macos")] {
            Ok(std::path::Path::new("/var/tmp/fig").join(name))
        } else if #[cfg(target_os = "windows")] {
            Ok(dirs::data_local_dir()
                .ok_or(DirectoryError::NoHomeDirectory)?
                .join("Fig")
                .join(name))
        }
    }
}

/// The desktop app socket path
///
/// - Linux/MacOS: `/var/tmp/fig/$USER/fig.socket`
/// - Windows: `%APPDATA%/Fig/fig.sock`
pub fn fig_socket_path() -> Result<PathBuf> {
    Ok(fig_ephemeral_dir()?.join("fig.socket"))
}

/// Get path to the daemon socket
///
/// - Linux/MacOS: `/var/tmp/fig/$USERNAME/daemon.socket`
/// - Windows: `%LOCALAPPDATA%\Fig\daemon.socket`
pub fn daemon_socket_path() -> Result<PathBuf> {
    Ok(fig_ephemeral_dir()?.join("daemon.socket"))
}

/// The path to secure socket
///
/// - Linux/MacOS: `/var/tmp/fig/$USER/secure.socket`
/// - Windows: `%APPDATA%/Fig/secure.sock`
pub fn secure_socket_path() -> Result<PathBuf> {
    if let Ok(parent) = env::var("FIG_PARENT") {
        parent_socket_path(whoami::username(), &parent)
    } else {
        Ok(fig_ephemeral_dir()?.join("secure.socket"))
    }
}

pub fn parent_socket_path(user_name: String, parent: impl Display) -> Result<PathBuf> {
    Ok(named_fig_ephemeral_dir(user_name)?
        .join("parent")
        .join(format!("{parent}.socket")))
}

/// Get path to a figterm socket
///
/// - Linux/Macos: `/var/tmp/fig/%USERNAME%/figterm/$SESSION_ID.socket`
/// - Windows: `%APPDATA%\Fig\$SESSION_ID.socket`
pub fn figterm_socket_path(session_id: impl Display) -> Result<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "windows")] {
            dirs::data_local_dir().map(|path| path.join("Fig").join(format!("figterm-{session_id}.socket"))).ok_or(DirectoryError::NoHomeDirectory)
        } else {
            Ok(fig_ephemeral_dir()?.join("figterm").join(format!("{session_id}.socket")))
        }
    }
}

/// Get path to "/usr/share/fig/manifest.json"
pub fn manifest_path() -> Result<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "linux")] {
            Ok(std::path::Path::new("/usr/share/fig/manifest.json").into())
        } else {
            panic!("This platform does not support build manifests")
        }
    }
}

/// Path to the managed binary directory
///
/// Note this is not implemented on Linux or MacOS
pub fn managed_binaries_dir() -> Result<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(any(target_os = "linux", target_os = "macos"))] {
            todo!();
        } else if #[cfg(target_os = "windows")] {
            Ok(fig_dir()?.join("bin"))
        }
    }
}

/// The path to the managed fig cli binary
///
/// Note this is not implemented on Linux or MacOS
pub fn managed_fig_cli_path() -> Result<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(any(target_os = "linux", target_os = "macos"))] {
            todo!();
        } else if #[cfg(target_os = "windows")] {
            Ok(managed_binaries_dir()?.join("fig.exe"))
        }
    }
}

macro_rules! utf8_dir {
    ($name:ident, $($arg:ident: $type:ty),*) => {
        paste::paste! {
            pub fn [<$name _utf8>]($($arg: $type),*) -> Result<Utf8PathBuf> {
                Ok($name($($arg),*)?.try_into()?)
            }
        }
    };
    ($name:ident) => {
        utf8_dir!($name,);
    };
}

utf8_dir!(home_dir);
utf8_dir!(fig_dir);
utf8_dir!(fig_data_dir);
utf8_dir!(fig_ephemeral_dir);
utf8_dir!(secure_socket_path);
utf8_dir!(named_fig_ephemeral_dir, name: String);
utf8_dir!(parent_socket_path, user_name: String, parent: impl Display);
utf8_dir!(figterm_socket_path, session_id: impl Display);
utf8_dir!(daemon_socket_path);
utf8_dir!(manifest_path);
utf8_dir!(managed_binaries_dir);
utf8_dir!(managed_fig_cli_path);

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
