use std::convert::TryInto;
use std::fmt::Display;
use std::path::{
    Path,
    PathBuf,
};

use camino::Utf8PathBuf;
use thiserror::Error;
use time::OffsetDateTime;

#[cfg(target_os = "macos")]
use crate::consts::CODEWHISPERER_CLI_BINARY_NAME;
use crate::system_info::is_remote;

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
    debug_env_binding!("FIG_DIRECTORIES_HOME_DIR");
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
    debug_env_binding!("FIG_DIRECTORIES_FIG_DATA_DIR");

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
    debug_env_binding!("FIG_DIRECTORIES_CACHE_DIR");

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

#[cfg(unix)]
pub fn root_socket_dir() -> &'static Path {
    Path::new("/var/tmp/codewhisperer")
}

/// The codewhisperer sockets directory of the local codewhisperer installation
///
/// - Linux: /var/tmp/codewhisperer/Alice
/// - MacOS: /var/tmp/codewhisperer/Alice
pub fn sockets_dir() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_SOCKETS_DIR");

    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            Ok(root_socket_dir().join(whoami::username()))
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
/// - Linux: /var/tmp/codewhisperer/Alice
/// - MacOS: /var/tmp/codewhisperer/Alice
pub fn host_sockets_dir() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_HOST_SOCKETS_DIR");

    #[cfg(target_os = "linux")]
    if crate::system_info::in_wsl() {
        use std::ffi::OsStr;
        use std::os::unix::prelude::OsStrExt;
        use std::process::Command;

        use bstr::ByteSlice;

        let socket_dir = Command::new("fig.exe").args(["_", "sockets-dir"]).output()?;
        let wsl_socket = Command::new("wslpath")
            .arg(OsStr::from_bytes(socket_dir.stdout.trim()))
            .output()?;
        return Ok(PathBuf::from(OsStr::from_bytes(wsl_socket.stdout.trim())));
    }

    sockets_dir()
}

/// Path to the managed binaries directory
///
/// - Linux: UNIMPLEMENTED
/// - MacOS: UNIMPLEMENTED
/// - Windows: %LOCALAPPDATA%\Fig\bin
pub fn managed_binaries_dir() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_MANAGED_BINARIES_DIR");

    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            // TODO: use fig_app_bundle() here!
            todo!();
        } else if #[cfg(target_os = "linux")] {
            todo!();
        } else if #[cfg(target_os = "windows")] {
            Ok(fig_dir()?.join("bin"))
        } else {
            todo!();
        }
    }
}

/// The path to all of the themes
pub fn themes_dir() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_THEMES_DIR");

    Ok(resources_path()?.join("themes"))
}

/// The autocomplete directory
pub fn autocomplete_dir() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_AUTOCOMPLETE_DIR");
    Ok(fig_data_dir()?.join("autocomplete"))
}

/// The autocomplete specs directory
pub fn autocomplete_specs_dir() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_AUTOCOMPLETE_SPECS_DIR");
    Ok(autocomplete_dir()?.join("specs"))
}

/// The path to the cloned repo containing the themes
pub fn themes_repo_dir() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_THEMES_REPO_DIR");
    Ok(fig_data_dir()?.join("themes"))
}

/// The directory to all the fig logs
/// - Linux: `/tmp/fig/$USER/logs`
/// - MacOS: `~/.fig/logs`
/// - Windows: `%TEMP%\fig\logs`
pub fn logs_dir() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_LOGS_DIR");
    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            Ok(std::env::temp_dir().join("codewhisperer").join(whoami::username()).join("logs"))
        } else if #[cfg(windows)] {
            Ok(std::env::temp_dir().join("codewhisperer").join("logs"))
        }
    }
}

/// The directory where fig places all data-sensitive backups
pub fn backups_dir() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_BACKUPS_DIR");

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
    debug_env_binding!("FIG_DIRECTORIES_UTC_BACKUP_DIR");

    let now = OffsetDateTime::now_utc().format(time::macros::format_description!(
        "[year]-[month]-[day]_[hour]-[minute]-[second]"
    ))?;

    Ok(backups_dir()?.join(now))
}

/// The directory where cached scripts are stored
pub fn scripts_cache_dir() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_SCRIPTS_CACHE_DIR");
    Ok(cache_dir()?.join("scripts"))
}

/// The desktop app socket path
///
/// - Linux/MacOS: `/var/tmp/fig/$USER/fig.socket`
/// - Windows: `%APPDATA%/Fig/fig.sock`
pub fn fig_socket_path() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_FIG_SOCKET_PATH");
    Ok(host_sockets_dir()?.join("fig.socket"))
}

/// The path to remote socket
///
/// - Linux/MacOS on ssh: `/var/tmp/fig-parent-$USER.socket`
/// - Linux/MacOS not on ssh: `/var/tmp/fig/$USER/secure.socket`
/// - Windows: `%APPDATA%/Fig/%USER%/secure.sock`
pub fn remote_socket_path() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_REMOTE_SOCKET_PATH");
    if is_remote() {
        if let Ok(parent_id) = std::env::var("FIG_PARENT") {
            if !parent_id.is_empty() {
                return parent_socket_path(&parent_id);
            }
        }
    }
    local_remote_socket_path()
}

/// The path to fig parent socket
///
/// - Linux/MacOS: `/var/tmp/fig-parent-$FIG_PARENT.socket`
/// - Windows: unused
pub fn parent_socket_path(parent_id: &str) -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_PARENT_SOCKET_PATH");
    Ok(Path::new(&format!("/var/tmp/fig-parent-{parent_id}.socket")).to_path_buf())
}

/// The path to local remote socket
///
/// - Linux/MacOS: `/var/tmp/fig/$USER/secure.socket`
/// - Windows: `%APPDATA%/Fig/%USER%/secure.sock`
pub fn local_remote_socket_path() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_LOCAL_REMOTE_SOCKET_PATH");
    Ok(host_sockets_dir()?.join("secure.socket"))
}

/// Get path to a figterm socket
///
/// - Linux/Macos: `/var/tmp/fig/%USERNAME%/figterm/$SESSION_ID.socket`
/// - Windows: `%APPDATA%\Fig\$SESSION_ID.socket`
pub fn figterm_socket_path(session_id: impl Display) -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_FIGTERM_SOCKET_PATH");
    Ok(sockets_dir()?.join("cwterm").join(format!("{session_id}.socket")))
}

/// The path to the resources directory
///
/// - MacOS: "/Applications/CodeWhisperer.app/Contents/Resources"
/// - Linux: "/usr/share/fig"
pub fn resources_path() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_RESOURCES_PATH");

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
    debug_env_binding!("FIG_DIRECTORIES_MANIFEST_PATH");

    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            Ok(resources_path()?.join("manifest.json"))
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
        if #[cfg(unix)] {
            todo!();
        } else if #[cfg(windows)] {
            Ok(managed_binaries_dir()?.join("fig.exe"))
        }
    }
}

/// The path to the fig settings file
pub fn settings_path() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_SETTINGS_PATH");
    Ok(fig_data_dir()?.join("settings.json"))
}

/// The path to the fig state file
pub fn state_path() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_STATE_PATH");
    Ok(fig_data_dir()?.join("state.json"))
}

/// The path to the lock file used to indicate that the app is updating
pub fn update_lock_path() -> Result<PathBuf> {
    debug_env_binding!("FIG_DIRECTORIES_UPDATE_LOCK_PATH");

    let data_dir = fig_data_dir()?;
    Ok(data_dir.join("update.lock"))
}

/// Path to the main credentials file
pub fn credentials_path() -> Result<PathBuf> {
    Ok(fig_data_dir()?.join("credentials.json"))
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
utf8_dir!(managed_binaries_dir);
utf8_dir!(managed_fig_cli_path);
utf8_dir!(backups_dir);
utf8_dir!(logs_dir);
utf8_dir!(ssh_saved_identities);
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

        test_environment_path!(home_dir, "FIG_DIRECTORIES_HOME_DIR");
        test_environment_path!(fig_data_dir, "FIG_DIRECTORIES_FIG_DATA_DIR");
        test_environment_path!(sockets_dir, "FIG_DIRECTORIES_SOCKETS_DIR");
        test_environment_path!(host_sockets_dir, "FIG_DIRECTORIES_HOST_SOCKETS_DIR");
        test_environment_path!(managed_binaries_dir, "FIG_DIRECTORIES_MANAGED_BINARIES_DIR");
        test_environment_path!(themes_dir, "FIG_DIRECTORIES_THEMES_DIR");
        test_environment_path!(themes_repo_dir, "FIG_DIRECTORIES_THEMES_REPO_DIR");
        test_environment_path!(logs_dir, "FIG_DIRECTORIES_LOGS_DIR");
        test_environment_path!(backups_dir, "FIG_DIRECTORIES_BACKUPS_DIR");
        test_environment_path!(utc_backup_dir, "FIG_DIRECTORIES_UTC_BACKUP_DIR");
        test_environment_path!(scripts_cache_dir, "FIG_DIRECTORIES_SCRIPTS_CACHE_DIR");
        test_environment_path!(fig_socket_path, "FIG_DIRECTORIES_FIG_SOCKET_PATH");
        test_environment_path!(manifest_path, "FIG_DIRECTORIES_MANIFEST_PATH");
        test_environment_path!(managed_fig_cli_path, "FIG_DIRECTORIES_MANAGED_FIG_CLI_PATH");
        test_environment_path!(settings_path, "FIG_DIRECTORIES_SETTINGS_PATH");
        test_environment_path!(state_path, "FIG_DIRECTORIES_STATE_PATH");
        test_environment_path!(ssh_saved_identities, "FIG_DIRECTORIES_SSH_SAVED_IDENTITIES");
    }
}

#[cfg(test)]
mod tests {
    use insta;

    use super::*;

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
        let user = whoami::username();
        path.ok()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
            .replace(&user, "$USER")
    }

    #[ignore]
    #[test]
    fn _snapshot_fig_data_dir() {
        linux!(fig_data_dir(), @"/home/$USER/.local/share/fig");
        macos!(fig_data_dir(), @"/Users/$USER/Library/Application Support/fig");
        windows!(fig_data_dir(), @r"C:\Users\$USER\AppData\Local\Fig\userdata");
    }

    #[ignore]
    #[test]
    fn _snapshot_sockets_dir() {
        linux!(sockets_dir(), @"/var/tmp/fig/$USER");
        macos!(sockets_dir(), @"/var/tmp/fig/$USER");
        windows!(sockets_dir(), @r"C:\Users\$USER\AppData\Local\Fig\sockets");
    }

    #[ignore]
    #[test]
    fn _snapshot_themes_dir() {
        linux!(themes_dir(), @"/home/$USER/.local/share/fig/themes/themes");
        macos!(themes_dir(), @"/Users/$USER/Library/Application Support/fig/themes/themes");
        windows!(themes_dir(), @r"C:\Users\$USER\AppData\Local\Fig\userdata\themes\themes");
    }

    #[ignore]
    #[test]
    fn _snapshot_themes_repo_dir() {
        linux!(themes_repo_dir(), @"/home/$USER/.local/share/fig/themes");
        macos!(themes_repo_dir(), @"/Users/$USER/Library/Application Support/fig/themes");
        windows!(themes_repo_dir(), @r"C:\Users\$USER\AppData\Local\Fig\userdata\themes");
    }

    #[ignore]
    #[test]
    fn _snapshot_backups_dir() {
        linux!(backups_dir(), @"/home/$USER/.fig.dotfiles.bak");
        macos!(backups_dir(), @"/Users/$USER/.fig.dotfiles.bak");
        windows!(backups_dir(), @r"C:\Users\$USER\AppData\Local\Fig\userdata\backups");
    }

    #[ignore]
    #[test]
    fn _snapshot_fig_socket_path() {
        linux!(fig_socket_path(), @"/var/tmp/fig/$USER/fig.socket");
        macos!(fig_socket_path(), @"/var/tmp/fig/$USER/fig.socket");
        windows!(fig_socket_path(), @r"C:\Users\$USER\AppData\Local\Fig\sockets\fig.socket");
    }

    #[ignore]
    #[test]
    fn _snapshot_remote_socket_path() {
        linux!(remote_socket_path(), @"/var/tmp/fig/$USER/secure.socket");
        macos!(remote_socket_path(), @"/var/tmp/fig/$USER/secure.socket");
        windows!(remote_socket_path(), @r"C:\Users\$USER\AppData\Local\Fig\sockets\secure.socket");
    }

    #[ignore]
    #[test]
    fn _snapshot_parent_socket_path() {
        linux!(parent_socket_path("$FIG_PARENT"), @"/var/tmp/fig-parent-$FIG_PARENT.socket");
        macos!(parent_socket_path("$FIG_PARENT"), @"/var/tmp/fig-parent-$FIG_PARENT.socket");
        // windows does not have a parent socket
    }

    #[ignore]
    #[test]
    fn _snapshot_local_remote_socket_path() {
        linux!(local_remote_socket_path(), @"/var/tmp/fig/$USER/secure.socket");
        macos!(local_remote_socket_path(), @"/var/tmp/fig/$USER/secure.socket");
        windows!(local_remote_socket_path(), @r"C:\Users\$USER\AppData\Local\Fig\sockets\secure.socket");
    }

    #[ignore]
    #[test]
    fn _snapshot_figterm_socket_path() {
        linux!(figterm_socket_path("$SESSION_ID"), @"/var/tmp/fig/$USER/figterm/$SESSION_ID.socket");
        macos!(figterm_socket_path("$SESSION_ID"), @"/var/tmp/fig/$USER/figterm/$SESSION_ID.socket");
        windows!(figterm_socket_path("$SESSION_ID"), @r"C:\Users\$USER\AppData\Local\Fig\sockets\figterm\$SESSION_ID.socket");
    }

    #[ignore]
    #[test]
    fn _snapshot_settings_path() {
        linux!(settings_path(), @"/home/$USER/.fig/settings.json");
        macos!(settings_path(), @"/Users/$USER/.fig/settings.json");
        windows!(settings_path(), @r"C:\Users\$USER\AppData\Lcoal\Fig\settings.json");
    }

    #[ignore]
    #[test]
    fn _snapshot_state_path() {
        linux!(state_path(), @"/home/$USER/.local/share/fig/state.json");
        macos!(state_path(), @"/Users/$USER/Library/Application Support/fig/state.json");
        windows!(state_path(), @r"C:\Users\$USER\AppData\Local\Fig\userdata\state.json");
    }

    #[ignore]
    #[test]
    fn _snapshot_update_lock_path() {
        linux!(update_lock_path(), @"/home/$USER/.local/share/fig/update.lock");
        macos!(update_lock_path(), @"/Users/$USER/Library/Application Support/fig/update.lock");
        windows!(update_lock_path(), @r"C:\Users\$USER\AppData\Local\Fig\userdata\update.lock");
    }

    #[ignore]
    #[test]
    fn _snapshot_credentials_path() {
        linux!(credentials_path(), @"/home/$USER/.local/share/fig/credentials.json");
        macos!(credentials_path(), @"/Users/$USER/Library/Application Support/fig/credentials.json");
        windows!(credentials_path(), @r"C:\Users\$USER\AppData\Local\Fig\userdata\credentials.json");
    }

    #[ignore]
    #[test]
    fn _snapshot_ssh_saved_identities() {
        linux!(ssh_saved_identities(), @"/home/$USER/.local/share/fig/access/ssh_saved_identities");
        macos!(ssh_saved_identities(), @"/Users/$USER/Library/Application Support/fig/access/ssh_saved_identities");
        windows!(ssh_saved_identities(), @r"C:\Users\$USER\AppData\Local\Fig\userdata\access\ssh_saved_identities");
    }
}
