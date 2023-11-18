use std::convert::TryInto;
use std::fmt::Display;
use std::path::PathBuf;

use camino::Utf8PathBuf;
use thiserror::Error;
use time::OffsetDateTime;

#[cfg(target_os = "macos")]
use crate::consts::CODEWHISPERER_CLI_BINARY_NAME;
// use crate::system_info::is_remote;

macro_rules! debug_env_binding {
    ($path:literal) => {
        #[cfg(any(debug_assertions, test))]
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
    #[error("runtime directory not found: neither XDG_RUNTIME_DIR nor TMPDIR were found")]
    NoRuntimeDirectory,
    #[error("non absolute path: {0:?}")]
    NonAbsolutePath(PathBuf),
    #[error("file does not exist: {0:?}")]
    FileDoesNotExist(PathBuf),
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
    debug_env_binding!("CW_DIRECTORIES_HOME_DIR");
    dirs::home_dir().ok_or(DirectoryError::NoHomeDirectory)
}

/// The config directory
///
/// - Linux: `$XDG_CONCW_HOME` or `$HOME/.config`
/// - MacOS: `$HOME/Library/Application Support`
/// - Windows: `{FOLDERID_RoamingAppData}`
pub fn config_dir() -> Result<PathBuf> {
    dirs::config_dir().ok_or(DirectoryError::NoHomeDirectory)
}

/// The codewhisperer data directory
///
/// - Linux: `$XDG_DATA_HOME/codewhisperer` or `$HOME/.local/share/codewhisperer`
/// - MacOS: `$HOME/Library/Application Support/codewhisperer`
pub fn fig_data_dir() -> Result<PathBuf> {
    debug_env_binding!("CW_DIRECTORIES_CW_DATA_DIR");

    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            Ok(dirs::data_local_dir()
                .ok_or(DirectoryError::NoHomeDirectory)?
                .join("codewhisperer"))
        } else if #[cfg(windows)] {
            Ok(fig_dir()?.join("userdata"))
        }
    }
}

/// The codewhisperer cache directory
///
/// - Linux: `$XDG_CACHE_HOME/codewhisperer` or `$HOME/.cache/codewhisperer`
/// - MacOS: `$HOME/Library/Caches/codewhisperer`
pub fn cache_dir() -> Result<PathBuf> {
    debug_env_binding!("CW_DIRECTORIES_CACHE_DIR");

    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            Ok(dirs::cache_dir()
                .ok_or(DirectoryError::NoHomeDirectory)?
                .join("codewhisperer"))
        } else if #[cfg(windows)] {
            Ok(fig_dir()?.join("cache"))
        }
    }
}

/// Runtime dir is used for runtime data that should not be persisted for a long time, e.g. socket
/// files and logs
///
/// The XDG_RUNTIME_DIR is set by systemd <https://www.freedesktop.org/software/systemd/man/latest/file-hierarchy.html#/run/user/>,
/// if this is not set such as on macOS it will fallback to TMPDIR which is secure on macOS
#[cfg(unix)]
fn runtime_dir() -> Result<PathBuf> {
    dirs::runtime_dir()
        .or_else(|| std::env::var_os("TMPDIR").map(PathBuf::from))
        .ok_or(DirectoryError::NoRuntimeDirectory)
}

/// The codewhisperer sockets directory of the local codewhisperer installation
///
/// - Linux: $XDG_RUNTIME_DIR/cwsock
/// - MacOS: $TMPDIR/cwsock
pub fn sockets_dir() -> Result<PathBuf> {
    debug_env_binding!("CW_DIRECTORIES_SOCKETS_DIR");

    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            Ok(runtime_dir()?.join("cwrun"))
        } else if #[cfg(windows)] {
            Ok(fig_dir()?.join("sockets"))
        }
    }
}

/// The directory on the host machine where socket files are stored
///
/// In WSL, this will correctly return the host machine socket path.
/// In other remote environments, it returns the same as `sockets_dir`
///
/// - Linux: $XDG_RUNTIME_DIR/cwsock
/// - MacOS: $TMPDIR/cwsock
pub fn host_sockets_dir() -> Result<PathBuf> {
    debug_env_binding!("CW_DIRECTORIES_HOST_SOCKETS_DIR");

    // TODO: make this work again
    // #[cfg(target_os = "linux")]
    // if crate::system_info::in_wsl() {
    //     use std::ffi::OsStr;
    //     use std::os::unix::prelude::OsStrExt;
    //     use std::process::Command;

    //     use bstr::ByteSlice;

    //     let socket_dir = Command::new("fig.exe").args(["_", "sockets-dir"]).output()?;
    //     let wsl_socket = Command::new("wslpath")
    //         .arg(OsStr::from_bytes(socket_dir.stdout.trim()))
    //         .output()?;
    //     return Ok(PathBuf::from(OsStr::from_bytes(wsl_socket.stdout.trim())));
    // }

    sockets_dir()
}

// Path to the managed binaries directory
//
// - Linux: UNIMPLEMENTED
// - MacOS: UNIMPLEMENTED
// - Windows: %LOCALAPPDATA%\Fig\bin
// pub fn _managed_binaries_dir() -> Result<PathBuf> {
//     debug_env_binding!("CW_DIRECTORIES_MANAGED_BINARIES_DIR");

//     cfg_if::cfg_if! {
//         if #[cfg(target_os = "macos")] {
//             // TODO: use fig_app_bundle() here!
//             todo!();
//         } else if #[cfg(target_os = "linux")] {
//             todo!();
//         } else if #[cfg(target_os = "windows")] {
//             Ok(fig_dir()?.join("bin"))
//         } else {
//             todo!();
//         }
//     }
// }

/// The path to all of the themes
pub fn themes_dir() -> Result<PathBuf> {
    debug_env_binding!("CW_DIRECTORIES_THEMES_DIR");

    Ok(resources_path()?.join("themes"))
}

/// The autocomplete directory
pub fn autocomplete_dir() -> Result<PathBuf> {
    debug_env_binding!("CW_DIRECTORIES_AUTOCOMPLETE_DIR");
    Ok(fig_data_dir()?.join("autocomplete"))
}

/// The autocomplete specs directory
pub fn autocomplete_specs_dir() -> Result<PathBuf> {
    debug_env_binding!("CW_DIRECTORIES_AUTOCOMPLETE_SPECS_DIR");
    Ok(autocomplete_dir()?.join("specs"))
}

/// The directory to all the fig logs
/// - Linux: `/tmp/fig/$USER/logs`
/// - MacOS: `$TMPDIR/logs`
/// - Windows: `%TEMP%\fig\logs`
pub fn logs_dir() -> Result<PathBuf> {
    debug_env_binding!("CW_DIRECTORIES_LOGS_DIR");
    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            Ok(runtime_dir()?.join("cwlog"))
        } else if #[cfg(windows)] {
            Ok(std::env::temp_dir().join("codewhisperer").join("logs"))
        }
    }
}

/// The directory where fig places all data-sensitive backups
pub fn backups_dir() -> Result<PathBuf> {
    debug_env_binding!("CW_DIRECTORIES_BACKUPS_DIR");

    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            Ok(home_dir()?.join(".codewhisperer.dotfiles.bak"))
        } else if #[cfg(windows)] {
            Ok(fig_data_dir()?.join("backups"))
        }
    }
}

/// The directory for time based data-sensitive backups
///
/// NOTE: This changes every second and cannot be cached
pub fn utc_backup_dir() -> Result<PathBuf> {
    debug_env_binding!("CW_DIRECTORIES_UTC_BACKUP_DIR");

    let now = OffsetDateTime::now_utc().format(time::macros::format_description!(
        "[year]-[month]-[day]_[hour]-[minute]-[second]"
    ))?;

    Ok(backups_dir()?.join(now))
}

/// The directory where cached scripts are stored
pub fn scripts_cache_dir() -> Result<PathBuf> {
    debug_env_binding!("CW_DIRECTORIES_SCRIPTS_CACHE_DIR");
    Ok(cache_dir()?.join("scripts"))
}

/// The desktop app socket path
///
/// - MacOS: `$TMPDIR/cwrun/desktop.sock`
/// - Linux: `$XDG_RUNTIME_DIR/cwrun/desktop.sock`
/// - Windows: `%APPDATA%/Fig/desktop.sock`
pub fn desktop_socket_path() -> Result<PathBuf> {
    debug_env_binding!("CW_DIRECTORIES_CW_SOCKET_PATH");
    Ok(host_sockets_dir()?.join("desktop.sock"))
}

/// The path to remote socket
// - Linux/MacOS on ssh:
// - Linux/MacOS not on ssh:
/// - MacOS: `$TMPDIR/cwrun/remote.sock`
/// - Linux: `$XDG_RUNTIME_DIR/cwrun/remote.sock`
/// - Windows: `%APPDATA%/Fig/%USER%/remote.sock`
pub fn remote_socket_path() -> Result<PathBuf> {
    debug_env_binding!("CW_DIRECTORIES_REMOTE_SOCKET_PATH");
    // TODO: reenable remote cw
    // if is_remote() {
    //     if let Ok(parent_id) = std::env::var("CW_PARENT") {
    //         if !parent_id.is_empty() {
    //             return parent_socket_path(&parent_id);
    //         }
    //     }
    // }
    local_remote_socket_path()
}

// The path to fig parent socket
//
// - Linux/MacOS: `/var/tmp/fig-parent-$CW_PARENT.sock`
// - Windows: unused
// pub fn parent_socket_path(parent_id: &str) -> Result<PathBuf> {
//     debug_env_binding!("CW_DIRECTORIES_PARENT_SOCKET_PATH");
//     Ok(Path::new(&format!("/var/tmp/fig-parent-{parent_id}.sock")).to_path_buf())
// }

/// The path to local remote socket
///
/// - MacOS: `$TMPDIR/cwrun/desktop.sock`
/// - Linux: `$XDG_RUNTIME_DIR/cwrun/desktop.sock`
/// - Windows: `%APPDATA%/Fig/%USER%/remote.sock`
pub fn local_remote_socket_path() -> Result<PathBuf> {
    debug_env_binding!("CW_DIRECTORIES_LOCAL_REMOTE_SOCKET_PATH");
    Ok(host_sockets_dir()?.join("remote.sock"))
}

/// Get path to a figterm socket
///
/// - Linux/Macos: `/var/tmp/fig/%USERNAME%/figterm/$SESSION_ID.sock`
/// - MacOS: `$TMPDIR/cwrun/t/$SESSION_ID.sock`
/// - Linux: `$XDG_RUNTIME_DIR/cwrun/t/$SESSION_ID.sock`
/// - Windows: `%APPDATA%\Fig\$SESSION_ID.sock`
pub fn figterm_socket_path(session_id: impl Display) -> Result<PathBuf> {
    debug_env_binding!("CW_DIRECTORIES_FIGTERM_SOCKET_PATH");
    Ok(sockets_dir()?.join("t").join(format!("{session_id}.sock")))
}

/// The path to the resources directory
///
/// - MacOS: "/Applications/CodeWhisperer.app/Contents/Resources"
/// - Linux: "/usr/share/fig"
pub fn resources_path() -> Result<PathBuf> {
    debug_env_binding!("CW_DIRECTORIES_RESOURCES_PATH");

    cfg_if::cfg_if! {
        if #[cfg(all(unix, not(target_os = "macos")))] {
            Ok(std::path::Path::new("/usr/share/fig").into())
        } else if #[cfg(target_os = "macos")] {
            Ok(std::path::Path::new("/Applications/CodeWhisperer.app/Contents/Resources").into())
        }
    }
}

/// The path to the fig install manifest
///
/// - MacOS: "/Applications/CodeWhisperer.app/Contents/Resources/manifest.json"
/// - Linux: "/usr/share/fig/manifest.json"
pub fn manifest_path() -> Result<PathBuf> {
    debug_env_binding!("CW_DIRECTORIES_MANIFEST_PATH");

    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            Ok(resources_path()?.join("manifest.json"))
        } else if #[cfg(target_os = "windows")] {
            Ok(managed_binaries_dir()?.join("manifest.json"))
        }
    }
}

// The path to the managed fig cli binary
//
// Note this is not implemented on Linux or MacOS
// pub fn managed_cw_cli_path() -> Result<PathBuf> {
//     debug_env_binding!("CW_DIRECTORIES_MANAGED_CW_CLI_PATH");

//     cfg_if::cfg_if! {
//         if #[cfg(unix)] {
//             todo!();
//         } else if #[cfg(windows)] {
//             Ok(managed_binaries_dir()?.join("fig.exe"))
//         }
//     }
// }

/// The path to the fig settings file
pub fn settings_path() -> Result<PathBuf> {
    debug_env_binding!("CW_DIRECTORIES_SETTINGS_PATH");
    Ok(fig_data_dir()?.join("settings.json"))
}

/// The path to the lock file used to indicate that the app is updating
pub fn update_lock_path() -> Result<PathBuf> {
    debug_env_binding!("CW_DIRECTORIES_UPDATE_LOCK_PATH");

    let data_dir = fig_data_dir()?;
    Ok(data_dir.join("update.lock"))
}

/// Path to the main credentials file
pub fn credentials_path() -> Result<PathBuf> {
    Ok(fig_data_dir()?.join("credentials.json"))
}

/// The path to the cli, relative to the running binary
pub fn relative_cli_path() -> Result<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            let path = crate::current_exe_origin().unwrap().parent().unwrap().join(CODEWHISPERER_CLI_BINARY_NAME);
            if path.exists() {
                Ok(path)
            } else {
                Err(DirectoryError::FileDoesNotExist(path))
            }
        } else {
            Ok(std::path::Path::new("cw").into())
        }
    }
}

utf8_dir!(home_dir);
utf8_dir!(fig_data_dir);
utf8_dir!(sockets_dir);
utf8_dir!(remote_socket_path);
utf8_dir!(figterm_socket_path, session_id: impl Display);
utf8_dir!(manifest_path);
// utf8_dir!(managed_binaries_dir);
// utf8_dir!(managed_cw_cli_path);
utf8_dir!(backups_dir);
utf8_dir!(logs_dir);
utf8_dir!(relative_cli_path);

#[cfg(any(debug_assertions, test))]
fn map_env_dir(path: &std::ffi::OsStr) -> Result<PathBuf> {
    let path = std::path::Path::new(path);
    path.is_absolute()
        .then(|| path.to_path_buf())
        .ok_or_else(|| DirectoryError::NonAbsolutePath(path.to_owned()))
}

#[cfg(test)]
mod test {
    use super::*;

    #[ignore]
    #[fig_test::test]
    fn environment_paths() {
        macro_rules! test_environment_path {
            ($path:expr, $name:literal) => {
                #[cfg(unix)]
                let path = "/testing";
                #[cfg(windows)]
                let path = "C://testing";

                std::env::set_var($name, path);
                assert_eq!($path().unwrap(), PathBuf::from(path));
                std::env::remove_var($name);
            };
        }

        test_environment_path!(home_dir, "CW_DIRECTORIES_HOME_DIR");
        test_environment_path!(fig_data_dir, "CW_DIRECTORIES_CW_DATA_DIR");
        test_environment_path!(sockets_dir, "CW_DIRECTORIES_SOCKETS_DIR");
        test_environment_path!(host_sockets_dir, "CW_DIRECTORIES_HOST_SOCKETS_DIR");
        // test_environment_path!(managed_binaries_dir, "CW_DIRECTORIES_MANAGED_BINARIES_DIR");
        test_environment_path!(themes_dir, "CW_DIRECTORIES_THEMES_DIR");
        test_environment_path!(logs_dir, "CW_DIRECTORIES_LOGS_DIR");
        test_environment_path!(backups_dir, "CW_DIRECTORIES_BACKUPS_DIR");
        test_environment_path!(utc_backup_dir, "CW_DIRECTORIES_UTC_BACKUP_DIR");
        test_environment_path!(scripts_cache_dir, "CW_DIRECTORIES_SCRIPTS_CACHE_DIR");
        test_environment_path!(desktop_socket_path, "CW_DIRECTORIES_CW_SOCKET_PATH");
        test_environment_path!(manifest_path, "CW_DIRECTORIES_MANIFEST_PATH");
        // test_environment_path!(managed_cw_cli_path, "CW_DIRECTORIES_MANAGED_CW_CLI_PATH");
        test_environment_path!(settings_path, "CW_DIRECTORIES_SETTINGS_PATH");
    }
}

#[cfg(test)]
mod tests {
    use insta;

    use super::*;

    /// If this test fails then either of these paths were changed.
    ///
    /// Since we set the permissions of the parent of these paths, make sure they're in folders we
    /// own otherwise we will set permissions of directories we shouldn't
    #[test]
    fn test_socket_paths() {
        assert_eq!(host_sockets_dir().unwrap().file_name().unwrap(), "cwrun");
        assert_eq!(
            figterm_socket_path("").unwrap().parent().unwrap().file_name().unwrap(),
            "t"
        );
    }

    macro_rules! assert_directory {
        ($value:expr, @ $snapshot:literal) => {
            insta::assert_snapshot!(
                insta::_macro_support::ReferenceValue::Inline($snapshot),
                sanitized_directory_path($value),
                stringify!(sanitized_directory_path($value))
            )
        };
    }

    macro_rules! macos {
        ($value:expr, @$snapshot:literal) => {
            #[cfg(target_os = "macos")]
            assert_directory!($value, @$snapshot)
        };
    }

    macro_rules! linux {
        ($value:expr, @$snapshot:literal) => {
            #[cfg(target_os = "linux")]
            assert_directory!($value, @$snapshot)
        };
    }

    macro_rules! windows {
        ($value:expr, @$snapshot:literal) => {
            #[cfg(target_os = "windows")]
            assert_directory!($value, @$snapshot)
        };
    }

    fn sanitized_directory_path(path: Result<PathBuf>) -> String {
        let mut path = path.ok().unwrap().into_os_string().into_string().unwrap();

        if let Ok(home) = std::env::var("HOME") {
            let home = home.strip_suffix('/').unwrap_or(&home);
            path = path.replace(home, "$HOME");
        }

        let user = whoami::username();
        path = path.replace(&user, "$USER");

        if let Ok(tmpdir) = std::env::var("TMPDIR") {
            let tmpdir = tmpdir.strip_suffix('/').unwrap_or(&tmpdir);
            path = path.replace(tmpdir, "$TMPDIR");
        }

        if let Ok(xdg_runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            let xdg_runtime_dir = xdg_runtime_dir.strip_suffix('/').unwrap_or(&xdg_runtime_dir);
            path = path.replace(xdg_runtime_dir, "$XDG_RUNTIME_DIR");
        }

        path
    }

    #[test]
    fn snapshot_fig_data_dir() {
        linux!(fig_data_dir(), @"$USER/.local/share/codewhisperer");
        macos!(fig_data_dir(), @"$HOME/Library/Application Support/codewhisperer");
        windows!(fig_data_dir(), @r"C:\Users\$USER\AppData\Local\Fig\userdata");
    }

    #[test]
    fn snapshot_sockets_dir() {
        linux!(sockets_dir(), @"$XDG_RUNTIME_DIR/cwrun");
        macos!(sockets_dir(), @"$TMPDIR/cwrun");
        windows!(sockets_dir(), @r"C:\Users\$USER\AppData\Local\Fig\sockets");
    }

    #[test]
    fn snapshot_themes_dir() {
        linux!(themes_dir(), @"/home/$USER/.local/share/fig/themes/themes");
        macos!(themes_dir(), @"/Applications/CodeWhisperer.app/Contents/Resources/themes");
        windows!(themes_dir(), @r"C:\Users\$USER\AppData\Local\Fig\userdata\themes\themes");
    }

    #[test]
    fn snapshot_backups_dir() {
        linux!(backups_dir(), @"$HOME/.codewhisperer.dotfiles.bak");
        macos!(backups_dir(), @"$HOME/.codewhisperer.dotfiles.bak");
        windows!(backups_dir(), @r"C:\Users\$USER\AppData\Local\Fig\userdata\backups");
    }

    #[test]
    fn snapshot_fig_socket_path() {
        linux!(desktop_socket_path(), @"$XDG_RUNTIME_DIR/cwrun/desktop.sock");
        macos!(desktop_socket_path(), @"$TMPDIR/cwrun/desktop.sock");
        windows!(desktop_socket_path(), @r"C:\Users\$USER\AppData\Local\Fig\sockets\desktop.sock");
    }

    #[test]
    fn snapshot_remote_socket_path() {
        linux!(remote_socket_path(), @"$XDG_RUNTIME_DIR/cwrun/remote.sock");
        macos!(remote_socket_path(), @"$TMPDIR/cwrun/remote.sock");
        windows!(remote_socket_path(), @r"C:\Users\$USER\AppData\Local\Fig\sockets\remote.sock");
    }

    // #[test]
    // fn snapshot_parent_socket_path() {
    //     linux!(parent_socket_path("$CW_PARENT"), @"/var/tmp/fig-parent-$CW_PARENT.sock");
    //     macos!(parent_socket_path("$CW_PARENT"), @"/var/tmp/fig-parent-$CW_PARENT.sock");
    //     // windows does not have a parent socket
    // }

    #[test]
    fn snapshot_local_remote_socket_path() {
        linux!(local_remote_socket_path(), @"$XDG_RUNTIME_DIR/cwrun/remote.sock");
        macos!(local_remote_socket_path(), @"$TMPDIR/cwrun/remote.sock");
        windows!(local_remote_socket_path(), @r"C:\Users\$USER\AppData\Local\Fig\sockets\remote.sock");
    }

    #[test]
    fn snapshot_figterm_socket_path() {
        linux!(figterm_socket_path("$SESSION_ID"), @"$XDG_RUNTIME_DIR/cwrun/t/$SESSION_ID.sock");
        macos!(figterm_socket_path("$SESSION_ID"), @"$TMPDIR/cwrun/t/$SESSION_ID.sock");
        windows!(figterm_socket_path("$SESSION_ID"), @r"C:\Users\$USER\AppData\Local\Fig\sockets\figterm\$SESSION_ID.sock");
    }

    #[test]
    fn snapshot_settings_path() {
        linux!(settings_path(), @"$HOME/.local/share/codewhisperer/settings.json");
        macos!(settings_path(), @"$HOME/Library/Application Support/codewhisperer/settings.json");
        windows!(settings_path(), @r"C:\Users\$USER\AppData\Lcoal\Fig\settings.json");
    }

    #[test]
    fn snapshot_update_lock_path() {
        linux!(update_lock_path(), @"$HOME/.local/share/codewhisperer/update.lock");
        macos!(update_lock_path(), @"$HOME/Library/Application Support/codewhisperer/update.lock");
        windows!(update_lock_path(), @r"C:\Users\$USER\AppData\Local\Fig\userdata\update.lock");
    }

    #[test]
    fn snapshot_credentials_path() {
        linux!(credentials_path(), @"$HOME/.local/share/codewhisperer/credentials.json");
        macos!(credentials_path(), @"$HOME/Library/Application Support/codewhisperer/credentials.json");
        windows!(credentials_path(), @r"C:\Users\$USER\AppData\Local\Fig\userdata\credentials.json");
    }

    #[test]
    #[cfg(unix)]
    fn socket_path_length() {
        use std::os::unix::ffi::OsStrExt;
        /// Sockets are bounded at 100 bytes, why, because legacy compat
        const MAX_SOCKET_LEN: usize = 100;

        let uuid = uuid::Uuid::new_v4().simple().to_string();
        let cwterm_socket = figterm_socket_path(uuid.to_string()).unwrap();
        let cwterm_socket_bytes = cwterm_socket.as_os_str().as_bytes().len();
        assert!(cwterm_socket_bytes <= MAX_SOCKET_LEN);

        let fig_socket = desktop_socket_path().unwrap();
        let fig_socket_bytes = fig_socket.as_os_str().as_bytes().len();
        assert!(fig_socket_bytes <= MAX_SOCKET_LEN);

        let secure_socket = remote_socket_path().unwrap();
        let secure_socket_bytes = secure_socket.as_os_str().as_bytes().len();
        assert!(secure_socket_bytes <= MAX_SOCKET_LEN);
    }
}
