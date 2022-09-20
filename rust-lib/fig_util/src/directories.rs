use std::convert::TryInto;
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
    match std::env::var_os("FIG_DOT_DIR") {
        Some(dot_dir) => map_env_dir(&dot_dir),
        None => {
            cfg_if::cfg_if! {
                if #[cfg(any(target_os = "linux", target_os = "macos"))] {
                    dirs::home_dir()
                        .ok_or(DirectoryError::NoHomeDirectory)
                        .map(|p| p.join(".fig"))
                } else if #[cfg(target_os = "windows")] {
                    Ok(dirs::data_local_dir().ok_or(DirectoryError::NoHomeDirectory)?.join("Fig"))
                }
            }
        },
    }
}

/// The fig data directory
///
/// - Linux: `$XDG_DATA_HOME/fig or $HOME/.local/share/fig`
/// - MacOS: `$HOME/Library/Application Support/fig`
/// - Windows: `%APPDATA%/Fig/userdata`
pub fn fig_data_dir() -> Result<PathBuf> {
    match std::env::var_os("FIG_DATA_DIR") {
        Some(data_dir) => map_env_dir(&data_dir),
        None => {
            cfg_if::cfg_if! {
                if #[cfg(any(target_os = "linux", target_os = "macos"))] {
                    dirs::data_local_dir()
                        .map(|path| path.join("fig"))
                        .ok_or(DirectoryError::NoHomeDirectory)
                } else if #[cfg(target_os = "windows")] {
                    Ok(fig_dir()?.join("userdata"))
                }
            }
        },
    }
}

/// The ephermeral fig sockets directory
///
/// - Linux: /var/tmp/fig/$USER
/// - Windows: %LOCALAPPDATA%/Fig/sockets
pub fn sockets_dir() -> Result<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "linux")] {
            use std::path::Path;
            use std::process::Command;
            use std::os::unix::prelude::OsStrExt;
            use std::ffi::OsStr;

            match crate::system_info::in_wsl() {
                true => {
                    let socket_dir = Command::new("fig.exe").args(["_", "sockets-dir"]).output()?;
                    let wsl_socket = Command::new("wslpath").arg(OsStr::from_bytes(&socket_dir.stdout)).output()?;
                    Ok(PathBuf::from(OsStr::from_bytes(&wsl_socket.stdout)).join(whoami::username()))
                },
                false => Ok(Path::new("/var/tmp/fig").join(whoami::username()))
            }
        } else if #[cfg(target_os = "macos")] {
            Ok(std::path::Path::new("/var/tmp/fig").join(whoami::username()))
        } else if #[cfg(target_os = "windows")] {
            Ok(fig_dir()?.join("sockets"))
        }
    }
}

/// Path to the managed binaries directory
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

/// The path to all of the themes
pub fn themes_dir() -> Result<PathBuf> {
    Ok(themes_repo_dir()?.join("themes"))
}

/// The path to the cloned repo containing the themes
pub fn themes_repo_dir() -> Result<PathBuf> {
    Ok(fig_data_dir()?.join("themes"))
}

/// The path to the fig plugins
pub fn plugins_dir() -> Result<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(any(target_os = "linux", target_os = "windows"))] {
            Ok(fig_data_dir()?.join("plugins"))
        } else if #[cfg(target_os = "macos")] {
            home_dir().map(|dir| dir.join(".local").join("share").join("fig").join("plugins"))
        }
    }
}

/// The desktop app socket path
///
/// - Linux/MacOS: `/var/tmp/fig/$USER/fig.socket`
/// - Windows: `%APPDATA%/Fig/fig.sock`
pub fn fig_socket_path() -> Result<PathBuf> {
    Ok(sockets_dir()?.join("fig.socket"))
}

/// Get path to the daemon socket
///
/// - Linux/MacOS: `/var/tmp/fig/$USERNAME/daemon.socket`
/// - Windows: `%LOCALAPPDATA%\Fig\daemon.socket`
pub fn daemon_socket_path() -> Result<PathBuf> {
    Ok(sockets_dir()?.join("daemon.socket"))
}

/// The path to secure socket
///
/// - Linux/MacOS: `/var/tmp/fig/$USER/secure.socket`
/// - Windows: `%APPDATA%/Fig/%USER%/secure.sock`
pub fn secure_socket_path() -> Result<PathBuf> {
    Ok(sockets_dir()?.join("secure.socket"))
}

/// Get path to a figterm socket
///
/// - Linux/Macos: `/var/tmp/fig/%USERNAME%/figterm/$SESSION_ID.socket`
/// - Windows: `%APPDATA%\Fig\$SESSION_ID.socket`
pub fn figterm_socket_path(session_id: impl Display) -> Result<PathBuf> {
    Ok(sockets_dir()?.join("figterm").join(format!("{session_id}.socket")))
}

/// The path to the fig install manifest
///
/// - Linux: "/usr/share/fig/manifest.json"
/// - Windows: "%APPDATA%/Local/Fig/bin/manifest.json"
pub fn manifest_path() -> Result<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "linux")] {
            Ok(std::path::Path::new("/usr/share/fig/manifest.json").into())
        } else if #[cfg(target_os = "macos")] {
            panic!("This platform does not support build manifests")
        } else if #[cfg(target_os = "windows")] {
            Ok(managed_binaries_dir()?.join("manifest.json"))
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
utf8_dir!(sockets_dir);
utf8_dir!(secure_socket_path);
utf8_dir!(figterm_socket_path, session_id: impl Display);
utf8_dir!(daemon_socket_path);
utf8_dir!(manifest_path);
utf8_dir!(managed_binaries_dir);
utf8_dir!(managed_fig_cli_path);

#[cfg(test)]
mod test {
    use super::*;

    #[ignore]
    #[test]
    fn path_names() {
        cfg_if::cfg_if! {
            if #[cfg(any(target_os = "linux", target_os = "macos"))] {
                assert_eq!(fig_dir().unwrap().file_name().unwrap(), ".fig");
                assert_eq!(fig_data_dir().unwrap().file_name().unwrap(), "fig");
            } else if #[cfg(target_os = "windows")] {
                assert_eq!(fig_dir().unwrap().file_name().unwrap(), "Fig");
                assert_eq!(fig_data_dir().unwrap().file_name().unwrap(), "userdata");
            }
        }
    }

    #[ignore]
    #[test]
    fn environment_paths() {
        let dir = dirs::home_dir().unwrap();
        let name = dir.file_name().unwrap();

        std::env::set_var("FIG_DOT_DIR", &dir);
        std::env::set_var("FIG_DATA_DIR", &dir);

        assert_eq!(fig_dir().unwrap().file_name().unwrap(), name);
        assert_eq!(fig_data_dir().unwrap().file_name().unwrap(), name);

        std::env::set_var("FIG_DOT_DIR", "abc");
        std::env::set_var("FIG_DATA_DIR", "def");

        fig_dir().unwrap_err();
        fig_data_dir().unwrap_err();
    }
}
