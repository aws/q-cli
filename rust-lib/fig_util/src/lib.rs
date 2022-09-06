pub mod directories;
mod error;
pub mod metadata;
mod open;
pub mod process_info;
mod shell;
pub mod system_info;
pub mod terminal;

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
