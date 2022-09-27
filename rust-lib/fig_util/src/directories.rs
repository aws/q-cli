use std::convert::TryInto;
use std::fmt::Display;
use std::path::{
    Path,
    PathBuf,
};

use camino::Utf8PathBuf;
use thiserror::Error;
use time::OffsetDateTime;

macro_rules! debug_env_binding {
    ($path:literal) => {
        #[cfg(debug_assertions)]
        if let Some(dir) = std::env::var_os($path) {
            return map_env_dir(&dir);
        }
    };
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

#[derive(Debug, Error)]
pub enum DirectoryError {
    #[error("home directory not found")]
    NoHomeDirectory,
    #[error("non absolute path: {0:?}")]
    NonAbsolutePath(PathBuf),
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    TimeFormat(#[from] time::error::Format),
    #[error(transparent)]
    Utf8FromPath(#[from] camino::FromPathError),
    #[error(transparent)]
    Utf8FromPathBuf(#[from] camino::FromPathBufError),
}

type Result<T, E = DirectoryError> = std::result::Result<T, E>;

/// The directory of the users home
///
/// - Linux: /home/Alice
/// - MacOS: /Users/Alice
/// - Windows: C:\Users\Alice
pub fn home_dir() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_HOME_DIR");
    dirs::home_dir().ok_or(DirectoryError::NoHomeDirectory)
}

/// The fig directory
///
/// - Linux: /home/Alice/.fig
/// - MacOS: /Users/Alice/.fig
/// - Windows: C:\Users\Alice\AppData\Local\Fig
pub fn fig_dir() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_FIG_DIR");

    cfg_if::cfg_if! {
        if #[cfg(any(target_os = "linux", target_os = "macos"))] {
            Ok(home_dir()?.join(".fig"))
        } else if #[cfg(target_os = "windows")] {
            Ok(dirs::data_local_dir()
                .ok_or(DirectoryError::NoHomeDirectory)?
                .join("Fig"))
        }
    }
}

/// The fig data directory
///
/// - Linux: `$XDG_DATA_HOME/fig or $HOME/.local/share/fig`
/// - MacOS: `$HOME/Library/Application Support/fig`
/// - Windows: `%LOCALAPPDATA%/Fig/userdata`
pub fn fig_data_dir() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_FIG_DATA_DIR");

    cfg_if::cfg_if! {
        if #[cfg(any(target_os = "linux", target_os = "macos"))] {
            Ok(dirs::data_local_dir()
                .ok_or(DirectoryError::NoHomeDirectory)?
                .join("fig"))
        } else if #[cfg(target_os = "windows")] {
            Ok(fig_dir()?.join("userdata"))
        }
    }
}

/// The ephemeral fig sockets directory
///
/// - Linux: /var/tmp/fig/Alice
/// - MacOS: /var/tmp/fig/Alice
/// - Windows: %LOCALAPPDATA%/Fig/sockets
pub fn sockets_dir() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_SOCKETS_DIR");

    cfg_if::cfg_if! {
        if #[cfg(target_os = "linux")] {
            use std::path::Path;
            use std::process::Command;
            use std::os::unix::prelude::OsStrExt;
            use std::ffi::OsStr;
            use bstr::ByteSlice;

            match crate::system_info::in_wsl() {
                true => {
                    let socket_dir = Command::new("fig.exe").args(["_", "sockets-dir"]).output()?;
                    let wsl_socket = Command::new("wslpath").arg(OsStr::from_bytes(socket_dir.stdout.trim())).output()?;
                    Ok(PathBuf::from(OsStr::from_bytes(wsl_socket.stdout.trim())))
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
/// - Linux: UNIMPLEMENTED
/// - MacOS: UNIMPLEMENTED
/// - Windows: %LOCALAPPDATA%\Fig\bin
pub fn managed_binaries_dir() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_MANAGED_BINARIES_DIR");

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
    debug_env_binding!("FIG_DIRECTORIES_THEMES_DIR");

    cfg_if::cfg_if! {
        if #[cfg(any(target_os = "linux", target_os = "windows"))] {
            Ok(themes_repo_dir()?.join("themes"))
        } else if #[cfg(target_os = "macos")] {
            deprecated::legacy_themes_dir()
        }
    }
}

/// The path to the cloned repo containing the themes
pub fn themes_repo_dir() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_THEMES_REPO_DIR");
    Ok(fig_data_dir()?.join("themes"))
}

/// The path to the fig plugins
pub fn plugins_dir() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_PLUGINS_DIR");

    cfg_if::cfg_if! {
        if #[cfg(any(target_os = "linux", target_os = "windows"))] {
            Ok(fig_data_dir()?.join("plugins"))
        } else if #[cfg(target_os = "macos")] {
            home_dir().map(|dir| dir.join(".local").join("share").join("fig").join("plugins"))
        }
    }
}

/// The directory to all the fig logs
pub fn logs_dir() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_LOGS_DIR");
    Ok(fig_dir()?.join("logs"))
}

/// The directory where fig places all data-sensitive backups
pub fn backups_dir() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_BACKUPS_DIR");

    cfg_if::cfg_if! {
        if #[cfg(any(target_os = "linux", target_os = "macos"))] {
            Ok(home_dir()?.join(".fig.dotfiles.bak"))
        } else if #[cfg(target_os = "windows")] {
            Ok(fig_data_dir()?.join("backups"))
        }
    }
}

/// The directory for time based data-sensitive backups
///
/// NOTE: This changes every second and cannot be cached
pub fn utc_backup_dir() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_UTC_BACKUP_DIR");

    let now = OffsetDateTime::now_utc().format(time::macros::format_description!(
        "[year]-[month]-[day]_[hour]-[minute]-[second]"
    ))?;

    Ok(backups_dir()?.join(now))
}

/// The desktop app socket path
///
/// - Linux/MacOS: `/var/tmp/fig/$USER/fig.socket`
/// - Windows: `%APPDATA%/Fig/fig.sock`
pub fn fig_socket_path() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_FIG_SOCKET_PATH");
    Ok(sockets_dir()?.join("fig.socket"))
}

/// Get path to the daemon socket
///
/// - Linux/MacOS: `/var/tmp/fig/$USERNAME/daemon.socket`
/// - Windows: `%LOCALAPPDATA%\Fig\daemon.socket`
pub fn daemon_socket_path() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_DAEMON_SOCKET_PATH");
    Ok(sockets_dir()?.join("daemon.socket"))
}

/// The path to secure socket
///
/// - Linux/MacOS on ssh: `/var/tmp/fig-parent-$USER.socket`
/// - Linux/MacOS not on ssh: `/var/tmp/fig/$USER/secure.socket`
/// - Windows: `%APPDATA%/Fig/%USER%/secure.sock`
pub fn secure_socket_path() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_SECURE_SOCKET_PATH");
    if let Ok(parent_id) = std::env::var("FIG_PARENT") {
        if !parent_id.is_empty() {
            return parent_socket_path(&parent_id);
        }
    }
    local_secure_socket_path()
}

/// The path to fig parent socket
///
/// - Linux/MacOS: `/var/tmp/fig-parent-$FIG_PARENT.socket`
/// - Windows: unused
pub fn parent_socket_path(parent_id: &str) -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_PARENT_SOCKET_PATH");
    Ok(Path::new(&format!("/var/tmp/fig-parent-{}.socket", parent_id)).to_path_buf())
}

/// The path to local secure socket
///
/// - Linux/MacOS: `/var/tmp/fig/$USER/secure.socket`
/// - Windows: `%APPDATA%/Fig/%USER%/secure.sock`
pub fn local_secure_socket_path() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_LOCAL_SECURE_SOCKET_PATH");
    Ok(sockets_dir()?.join("secure.socket"))
}

/// Get path to a figterm socket
///
/// - Linux/Macos: `/var/tmp/fig/%USERNAME%/figterm/$SESSION_ID.socket`
/// - Windows: `%APPDATA%\Fig\$SESSION_ID.socket`
pub fn figterm_socket_path(session_id: impl Display) -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_FIGTERM_SOCKET_PATH");
    Ok(sockets_dir()?.join("figterm").join(format!("{session_id}.socket")))
}

/// The path to the fig install manifest
///
/// - Linux: "/usr/share/fig/manifest.json"
/// - Windows: "%LOCALAPPDATA%/Fig/bin/manifest.json"
pub fn manifest_path() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_MANIFEST_PATH");

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
    debug_env_binding!("FIG_DIRECTORIES_MANAGED_FIG_CLI_PATH");

    cfg_if::cfg_if! {
        if #[cfg(any(target_os = "linux", target_os = "macos"))] {
            todo!();
        } else if #[cfg(target_os = "windows")] {
            Ok(managed_binaries_dir()?.join("fig.exe"))
        }
    }
}

/// The path to the saved ssh identities file
///
/// - Linux: `$XDG_DATA_HOME/fig or $HOME/.local/share/fig/access/ssh_saved_identities`
/// - MacOS: `$HOME/Library/Application Support/fig/access/ssh_saved_identities`
/// - Windows: `%LOCALAPPDATA%/Fig/userdata/access/ssh_saved_identities`
pub fn ssh_saved_identities() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_SSH_SAVED_IDENTITIES");

    Ok(fig_data_dir()?.join("access").join("ssh_saved_identities"))
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
utf8_dir!(plugins_dir);
utf8_dir!(backups_dir);
utf8_dir!(logs_dir);
utf8_dir!(ssh_saved_identities);

fn map_env_dir(path: &std::ffi::OsStr) -> Result<PathBuf> {
    let path = std::path::Path::new(path);
    path.is_absolute()
        .then(|| path.to_path_buf())
        .ok_or_else(|| DirectoryError::NonAbsolutePath(path.to_owned()))
}

#[cfg(target_os = "macos")]
mod deprecated {
    use super::*;

    pub fn legacy_themes_dir() -> Result<PathBuf> {
        let new_theme_dir = themes_dir()?;
        match new_theme_dir.exists() {
            true => Ok(new_theme_dir),
            false => Ok(themes_repo_dir()?.join("themes")),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn path_names() {
        macro_rules! test_path_name {
            ($path:expr, $name:expr) => {
                assert_eq!(
                    $path().unwrap().file_name().unwrap().to_str().unwrap(),
                    $name
                );
            };
        }

        cfg_if::cfg_if! {
            if #[cfg(any(target_os = "linux", target_os = "macos"))] {
                test_path_name!(fig_dir, ".fig");
                test_path_name!(fig_data_dir, "fig");
                test_path_name!(sockets_dir, whoami::username());
                test_path_name!(backups_dir, ".fig.dotfiles.bak");
            } else if #[cfg(target_os = "windows")] {
                test_path_name!(fig_dir, "Fig");
                test_path_name!(fig_data_dir, "userdata");
                test_path_name!(sockets_dir, "sockets");
                test_path_name!(managed_binaries_dir, "bin");
                test_path_name!(backups_dir, "backups");
                test_path_name!(managed_fig_cli_path, "fig.exe");
            }
        }

        test_path_name!(themes_dir, "themes");
        test_path_name!(themes_repo_dir, "themes");
        test_path_name!(plugins_dir, "plugins");
        test_path_name!(logs_dir, "logs");
        test_path_name!(fig_socket_path, "fig.socket");
        test_path_name!(daemon_socket_path, "daemon.socket");
        test_path_name!(local_secure_socket_path, "secure.socket");
        test_path_name!(manifest_path, "manifest.json");
        test_path_name!(ssh_saved_identities, "ssh_saved_identities");
    }

    #[test]
    fn environment_paths() {
        macro_rules! test_environment_path {
            ($path:expr, $name:literal) => {
                #[cfg(any(target_os = "linux", target_os = "macos"))]
                let path = "/testing";
                #[cfg(target_os = "windows")]
                let path = "C://testing";

                std::env::set_var($name, path);
                assert_eq!($path().unwrap(), PathBuf::from(path));
                std::env::remove_var($name);
            };
        }

        test_environment_path!(home_dir, "FIG_DIRECTORIES_HOME_DIR");
        test_environment_path!(fig_dir, "FIG_DIRECTORIES_FIG_DIR");
        test_environment_path!(fig_data_dir, "FIG_DIRECTORIES_FIG_DATA_DIR");
        test_environment_path!(sockets_dir, "FIG_DIRECTORIES_SOCKETS_DIR");
        test_environment_path!(managed_binaries_dir, "FIG_DIRECTORIES_MANAGED_BINARIES_DIR");
        test_environment_path!(themes_dir, "FIG_DIRECTORIES_THEMES_DIR");
        test_environment_path!(themes_repo_dir, "FIG_DIRECTORIES_THEMES_REPO_DIR");
        test_environment_path!(plugins_dir, "FIG_DIRECTORIES_PLUGINS_DIR");
        test_environment_path!(logs_dir, "FIG_DIRECTORIES_LOGS_DIR");
        test_environment_path!(backups_dir, "FIG_DIRECTORIES_BACKUPS_DIR");
        test_environment_path!(utc_backup_dir, "FIG_DIRECTORIES_UTC_BACKUP_DIR");
        test_environment_path!(fig_socket_path, "FIG_DIRECTORIES_FIG_SOCKET_PATH");
        test_environment_path!(daemon_socket_path, "FIG_DIRECTORIES_DAEMON_SOCKET_PATH");
        test_environment_path!(manifest_path, "FIG_DIRECTORIES_MANIFEST_PATH");
        test_environment_path!(managed_fig_cli_path, "FIG_DIRECTORIES_MANAGED_FIG_CLI_PATH");
        test_environment_path!(ssh_saved_identities, "FIG_DIRECTORIES_SSH_SAVED_IDENTITIES");
    }
}
