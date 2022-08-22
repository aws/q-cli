pub mod directories;
mod error;
mod open;
pub mod process_info;
mod shell;
mod system_info;
pub mod terminal;
pub mod wsl;

pub use error::Error;
use once_cell::sync::Lazy;
pub use open::{
    open_url,
    open_url_async,
};
pub use process_info::get_parent_process_exe;
use rand::Rng;
pub use shell::Shell;
#[cfg(target_os = "linux")]
pub use system_info::{
    detect_desktop,
    get_linux_os_release,
    DesktopEnvironment,
    DisplayServer,
    LinuxOsRelease,
};
pub use system_info::{
    get_arch,
    get_platform,
    get_system_id,
};
pub use terminal::Terminal;

pub fn gen_hex_string() -> String {
    let mut buf = [0u8; 32];
    rand::thread_rng().fill(&mut buf);
    hex::encode(buf)
}

static IN_SSH: Lazy<bool> = Lazy::new(|| {
    std::env::var_os("SSH_CLIENT").is_some()
        || std::env::var_os("SSH_CONNECTION").is_some()
        || std::env::var_os("SSH_TTY").is_some()
});

pub fn in_ssh() -> bool {
    *IN_SSH
}
