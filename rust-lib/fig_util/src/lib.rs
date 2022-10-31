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

use std::path::{
    Path,
    PathBuf,
};

pub use open::{
    open_url,
    open_url_async,
};
pub use process_info::get_parent_process_exe;
use rand::Rng;
pub use shell::Shell;
pub use terminal::Terminal;
use thiserror::Error;

pub use crate::desktop::is_fig_desktop_running;

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
    #[error("failed to launch fig: `{0}`")]
    LaunchError(String),
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
pub fn fig_bundle() -> Option<PathBuf> {
    let current_exe = current_exe_origin().ok()?;

    // Verify we have .../Bundle.app/Contents/MacOS/binary-name
    let mut parts: PathBuf = current_exe.components().rev().skip(1).take(3).collect();
    parts = parts.iter().rev().collect();

    if parts != PathBuf::from("Fig.app/Contents/MacOS") {
        return None;
    }

    // .../Bundle.app/Contents/MacOS/binary-name -> .../Bundle.app
    current_exe.ancestors().nth(3).map(|s| s.into())
}
