use std::convert::TryInto;
use std::fmt::Display;
use std::path::PathBuf;

use camino::Utf8PathBuf;
use thiserror::Error;
use time::OffsetDateTime;

#[cfg(target_os = "macos")]
use crate::consts::CLI_BINARY_NAME;
use crate::system_info::is_remote;
use crate::RUNTIME_DIR_NAME;

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
    #[error(transparent)]
    FromVecWithNul(#[from] std::ffi::FromVecWithNulError),
    #[error(transparent)]
    IntoString(#[from] std::ffi::IntoStringError),
    #[error("Q_PARENT env variable not set")]
    QParentNotSet,
}

type Result<T, E = DirectoryError> = std::result::Result<T, E>;

/// The directory of the users home
///
/// - Linux: /home/Alice
/// - MacOS: /Users/Alice
/// - Windows: C:\Users\Alice
pub fn home_dir() -> Result<PathBuf> {
    dirs::home_dir().ok_or(DirectoryError::NoHomeDirectory)
}

/// The config directory
///
/// - Linux: `$XDG_CONFIG_HOME` or `$HOME/.config`
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

/// Get the macos tempdir from the `confstr` function
///
/// See: <https://man7.org/linux/man-pages/man3/confstr.3.html>
#[cfg(target_os = "macos")]
fn macos_tempdir() -> Result<PathBuf> {
    let len = unsafe { libc::confstr(libc::_CS_DARWIN_USER_TEMP_DIR, std::ptr::null::<i8>().cast_mut(), 0) };
    let mut buf: Vec<u8> = vec![0; len];
    unsafe { libc::confstr(libc::_CS_DARWIN_USER_TEMP_DIR, buf.as_mut_ptr().cast(), buf.len()) };
    let c_string = std::ffi::CString::from_vec_with_nul(buf)?;
    let str = c_string.into_string()?;
    Ok(PathBuf::from(str))
}

/// Runtime dir is used for runtime data that should not be persisted for a long time, e.g. socket
/// files and logs
///
/// The XDG_RUNTIME_DIR is set by systemd <https://www.freedesktop.org/software/systemd/man/latest/file-hierarchy.html#/run/user/>,
/// if this is not set such as on macOS it will fallback to TMPDIR which is secure on macOS
#[cfg(unix)]
fn runtime_dir() -> Result<PathBuf> {
    let mut dir = dirs::runtime_dir();
    dir = dir.or_else(|| std::env::var_os("TMPDIR").map(PathBuf::from));

    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            let macos_tempdir = macos_tempdir()?;
            dir = dir.or(Some(macos_tempdir));
        } else {
            dir = dir.or_else(|| Some(std::env::temp_dir()));
        }
    }

    dir.ok_or(DirectoryError::NoRuntimeDirectory)
}

/// The codewhisperer sockets directory of the local codewhisperer installation
///
/// - Linux: $XDG_RUNTIME_DIR/cwrun
/// - MacOS: $TMPDIR/cwrun
pub fn sockets_dir() -> Result<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            Ok(runtime_dir()?.join(RUNTIME_DIR_NAME))
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
/// - Linux: $XDG_RUNTIME_DIR/cwrun
/// - MacOS: $TMPDIR/cwrun
pub fn host_sockets_dir() -> Result<PathBuf> {
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

/// The path to all of the themes
pub fn themes_dir() -> Result<PathBuf> {
    Ok(resources_path()?.join("themes"))
}

/// The autocomplete directory
pub fn autocomplete_dir() -> Result<PathBuf> {
    Ok(fig_data_dir()?.join("autocomplete"))
}

/// The autocomplete specs directory
pub fn autocomplete_specs_dir() -> Result<PathBuf> {
    Ok(autocomplete_dir()?.join("specs"))
}

/// The directory to all the fig logs
/// - Linux: `/tmp/fig/$USER/logs`
/// - MacOS: `$TMPDIR/logs`
/// - Windows: `%TEMP%\fig\logs`
pub fn logs_dir() -> Result<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            use crate::CLI_BINARY_NAME;
            Ok(runtime_dir()?.join(format!("{CLI_BINARY_NAME}log")))
        } else if #[cfg(windows)] {
            Ok(std::env::temp_dir().join("codewhisperer").join("logs"))
        }
    }
}

/// The directory where fig places all data-sensitive backups
pub fn backups_dir() -> Result<PathBuf> {
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
    let now = OffsetDateTime::now_utc().format(time::macros::format_description!(
        "[year]-[month]-[day]_[hour]-[minute]-[second]"
    ))?;

    Ok(backups_dir()?.join(now))
}

/// The directory where cached scripts are stored
pub fn scripts_cache_dir() -> Result<PathBuf> {
    Ok(cache_dir()?.join("scripts"))
}

/// The desktop app socket path
///
/// - MacOS: `$TMPDIR/cwrun/desktop.sock`
/// - Linux: `$XDG_RUNTIME_DIR/cwrun/desktop.sock`
/// - Windows: `%APPDATA%/Fig/desktop.sock`
pub fn desktop_socket_path() -> Result<PathBuf> {
    Ok(host_sockets_dir()?.join("desktop.sock"))
}

/// The path to remote socket
// - Linux/MacOS on ssh: At the value of `Q_PARENT`
// - Linux/MacOS not on ssh:
/// - MacOS: `$TMPDIR/cwrun/remote.sock`
/// - Linux: `$XDG_RUNTIME_DIR/cwrun/remote.sock`
/// - Windows: `%APPDATA%/Fig/%USER%/remote.sock`
pub fn remote_socket_path() -> Result<PathBuf> {
    // TODO(grant): This is only enabled on Linux for now to prevent public dist
    if is_remote() && cfg!(target_os = "linux") {
        if let Some(parent_socket) = std::env::var_os("Q_PARENT") {
            Ok(PathBuf::from(parent_socket))
        } else {
            Err(DirectoryError::QParentNotSet)
        }
    } else {
        local_remote_socket_path()
    }
}

/// The path to local remote socket
///
/// - MacOS: `$TMPDIR/cwrun/desktop.sock`
/// - Linux: `$XDG_RUNTIME_DIR/cwrun/desktop.sock`
/// - Windows: `%APPDATA%/Fig/%USER%/remote.sock`
pub fn local_remote_socket_path() -> Result<PathBuf> {
    Ok(host_sockets_dir()?.join("remote.sock"))
}

/// Get path to a figterm socket
///
/// - Linux/Macos: `/var/tmp/fig/%USERNAME%/figterm/$SESSION_ID.sock`
/// - MacOS: `$TMPDIR/cwrun/t/$SESSION_ID.sock`
/// - Linux: `$XDG_RUNTIME_DIR/cwrun/t/$SESSION_ID.sock`
/// - Windows: `%APPDATA%\Fig\$SESSION_ID.sock`
pub fn figterm_socket_path(session_id: impl Display) -> Result<PathBuf> {
    Ok(sockets_dir()?.join("t").join(format!("{session_id}.sock")))
}

/// The path to the resources directory
///
/// - MacOS: "/Applications/Q.app/Contents/Resources"
/// - Linux: "/usr/share/fig"
pub fn resources_path() -> Result<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(all(unix, not(target_os = "macos")))] {
            Ok(std::path::Path::new("/usr/share/fig").into())
        } else if #[cfg(target_os = "macos")] {
            Ok(crate::app_bundle_path().join(crate::macos::BUNDLE_CONTENTS_RESOURCE_PATH))
        }
    }
}

/// The path to the fig install manifest
///
/// - MacOS: "/Applications/Q.app/Contents/Resources/manifest.json"
/// - Linux: "/usr/share/fig/manifest.json"
pub fn manifest_path() -> Result<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            Ok(resources_path()?.join("manifest.json"))
        } else if #[cfg(target_os = "windows")] {
            Ok(managed_binaries_dir()?.join("manifest.json"))
        }
    }
}

/// The path to the fig settings file
pub fn settings_path() -> Result<PathBuf> {
    Ok(fig_data_dir()?.join("settings.json"))
}

/// The path to the lock file used to indicate that the app is updating
pub fn update_lock_path() -> Result<PathBuf> {
    Ok(fig_data_dir()?.join("update.lock"))
}

/// Path to the main credentials file
pub fn credentials_path() -> Result<PathBuf> {
    Ok(fig_data_dir()?.join("credentials.json"))
}

/// The path to the cli, relative to the running binary
pub fn relative_cli_path() -> Result<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            let path = crate::current_exe_origin().unwrap().parent().unwrap().join(CLI_BINARY_NAME);
            if path.exists() {
                Ok(path)
            } else {
                Err(DirectoryError::FileDoesNotExist(path))
            }
        } else {
            Ok(std::path::Path::new(crate::CLI_BINARY_NAME).into())
        }
    }
}

utf8_dir!(home_dir);
utf8_dir!(fig_data_dir);
utf8_dir!(sockets_dir);
utf8_dir!(remote_socket_path);
utf8_dir!(figterm_socket_path, session_id: impl Display);
utf8_dir!(manifest_path);
utf8_dir!(backups_dir);
utf8_dir!(logs_dir);
utf8_dir!(relative_cli_path);

// TODO(grant): Add back path tests on linux
#[cfg(all(test, not(target_os = "linux")))]
mod tests {
    use insta;

    use super::*;

    /// If this test fails then either of these paths were changed.
    ///
    /// Since we set the permissions of the parent of these paths, make sure they're in folders we
    /// own otherwise we will set permissions of directories we shouldn't
    #[test]
    fn test_socket_paths() {
        assert_eq!(
            host_sockets_dir().unwrap().file_name().unwrap().to_str().unwrap(),
            format!("cwrun")
        );
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
        let mut path = path.unwrap().into_os_string().into_string().unwrap();

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

        #[cfg(target_os = "macos")]
        {
            if let Ok(tmpdir) = macos_tempdir() {
                let tmpdir = tmpdir.to_str().unwrap();
                let tmpdir = tmpdir.strip_suffix('/').unwrap_or(tmpdir);
                path = path.replace(tmpdir, "$TMPDIR");
            };
        }

        if let Ok(xdg_runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            let xdg_runtime_dir = xdg_runtime_dir.strip_suffix('/').unwrap_or(&xdg_runtime_dir);
            path = path.replace(xdg_runtime_dir, "$XDG_RUNTIME_DIR");
        }

        #[cfg(target_os = "linux")]
        {
            path = path.replace("/tmp", "$TMPDIR");
        }

        path
    }

    #[test]
    fn snapshot_fig_data_dir() {
        linux!(fig_data_dir(), @"$HOME/.local/share/codewhisperer");
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
        linux!(themes_dir(), @"/usr/share/fig/themes");
        macos!(themes_dir(), @"/Applications/Q.app/Contents/Resources/themes");
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
    //     linux!(parent_socket_path("$Q_PARENT"), @"/var/tmp/fig-parent-$Q_PARENT.sock");
    //     macos!(parent_socket_path("$Q_PARENT"), @"/var/tmp/fig-parent-$Q_PARENT.sock");
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
        let qterm_socket = figterm_socket_path(uuid.clone()).unwrap();
        let qterm_socket_bytes = qterm_socket.as_os_str().as_bytes().len();
        assert!(qterm_socket_bytes <= MAX_SOCKET_LEN);

        let fig_socket = desktop_socket_path().unwrap();
        let fig_socket_bytes = fig_socket.as_os_str().as_bytes().len();
        assert!(fig_socket_bytes <= MAX_SOCKET_LEN);

        let secure_socket = remote_socket_path().unwrap();
        let secure_socket_bytes = secure_socket.as_os_str().as_bytes().len();
        assert!(secure_socket_bytes <= MAX_SOCKET_LEN);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn macos_tempdir_test() {
        let tmpdir = macos_tempdir().unwrap();
        println!("{:?}", tmpdir);
    }
}
