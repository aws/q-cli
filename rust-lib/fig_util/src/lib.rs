pub mod directories;
mod error;
pub mod manifest;
mod open;
pub mod process_info;
mod shell;
pub mod system_info;
pub mod terminal;

use std::path::{
    Path,
    PathBuf,
};

pub use error::Error;
pub use open::{
    open_url,
    open_url_async,
};
pub use process_info::get_parent_process_exe;
use rand::Rng;
pub use shell::Shell;
pub use terminal::Terminal;

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
