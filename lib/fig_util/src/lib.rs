pub mod directories;
pub mod manifest;
mod open;
pub mod process_info;
mod shell;
pub mod system_info;
pub mod terminal;

pub mod consts;
pub mod desktop;
#[cfg(target_os = "macos")]
pub mod launchd_plist;

use std::cmp::Ordering;
use std::path::{
    Path,
    PathBuf,
};

pub use consts::*;
pub use open::{
    open_url,
    open_url_async,
};
pub use process_info::get_parent_process_exe;
use rand::Rng;
pub use shell::Shell;
pub use terminal::Terminal;
use thiserror::Error;

pub use crate::desktop::desktop_app_running;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io operation error")]
    IoError(#[from] std::io::Error),
    #[error("unsupported platform")]
    UnsupportedPlatform,
    #[error("unsupported architecture")]
    UnsupportedArch,
    #[error(transparent)]
    Directory(#[from] crate::directories::DirectoryError),
    #[error("process has no parent")]
    NoParentProcess,
    #[error("could not find the os hwid")]
    HwidNotFound,
    #[error("the shell, `{0}`, isn't supported yet")]
    UnknownShell(String),
    #[error("missing environment variable `{0}`")]
    MissingEnv(&'static str),
    #[error("unknown display server `{0}`")]
    UnknownDisplayServer(String),
    #[error("unknown desktop `{0}`")]
    UnknownDesktop(String),
    #[error("failed to launch: `{0}`")]
    LaunchError(String),
    #[error(transparent)]
    StrUtf8Error(#[from] std::str::Utf8Error),
    #[error("Failed to parse shell {0} version")]
    ShellVersion(Shell),
}

pub fn gen_hex_string() -> String {
    let mut buf = [0u8; 32];
    rand::thread_rng().fill(&mut buf);
    hex::encode(buf)
}

pub fn search_xdg_data_dirs(ext: impl AsRef<std::path::Path>) -> Option<PathBuf> {
    let ext = ext.as_ref();
    if let Ok(xdg_data_dirs) = std::env::var("XDG_DATA_DIRS") {
        for base in xdg_data_dirs.split(':') {
            let check = Path::new(base).join(ext);
            if check.exists() {
                return Some(check);
            }
        }
    }
    None
}

/// Returns the path to the original executable, not the symlink
pub fn current_exe_origin() -> Result<PathBuf, Error> {
    Ok(std::env::current_exe()?.canonicalize()?)
}

#[must_use]
#[cfg(target_os = "macos")]
fn app_bundle_path_opt() -> Option<PathBuf> {
    use consts::macos::BUNDLE_CONTENTS_MACOS_PATH;

    let current_exe = current_exe_origin().ok()?;

    // Verify we have .../Bundle.app/Contents/MacOS/binary-name
    let mut parts: PathBuf = current_exe.components().rev().skip(1).take(3).collect();
    parts = parts.iter().rev().collect();

    if parts != Path::new(APP_BUNDLE_NAME).join(BUNDLE_CONTENTS_MACOS_PATH) {
        return None;
    }

    // .../Bundle.app/Contents/MacOS/binary-name -> .../Bundle.app
    current_exe.ancestors().nth(3).map(|s| s.into())
}

#[must_use]
#[cfg(target_os = "macos")]
pub fn app_bundle_path() -> PathBuf {
    app_bundle_path_opt().unwrap_or_else(|| Path::new("/Applications").join(APP_BUNDLE_NAME))
}

pub fn partitioned_compare(lhs: &str, rhs: &str, by: char) -> Ordering {
    let sides = lhs
        .split(by)
        .filter(|x| !x.is_empty())
        .zip(rhs.split(by).filter(|x| !x.is_empty()));

    for (lhs, rhs) in sides {
        match if lhs.chars().all(|x| x.is_numeric()) && rhs.chars().all(|x| x.is_numeric()) {
            // perform a numerical comparison
            let lhs: u64 = lhs.parse().unwrap();
            let rhs: u64 = rhs.parse().unwrap();
            lhs.cmp(&rhs)
        } else {
            // perform a lexical comparison
            lhs.cmp(rhs)
        } {
            Ordering::Equal => continue,
            s => return s,
        }
    }

    lhs.len().cmp(&rhs.len())
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use crate::partitioned_compare;

    #[test]
    fn test_partitioned_compare() {
        assert_eq!(partitioned_compare("1.2.3", "1.2.3", '.'), Ordering::Equal);
        assert_eq!(partitioned_compare("1.2.3", "1.2.2", '.'), Ordering::Greater);
        assert_eq!(partitioned_compare("4-a-b", "4-a-c", '-'), Ordering::Less);
        assert_eq!(partitioned_compare("0?0?0", "0?0", '?'), Ordering::Greater);
    }
}
