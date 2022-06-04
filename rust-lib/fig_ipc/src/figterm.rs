use std::{path::{
    PathBuf,
}, fmt::Display};

use wsl::is_wsl;

pub fn get_figterm_socket_path(session_id: impl Display) -> PathBuf {
    match is_wsl() {
        true => PathBuf::from(format!("/mnt/c/fig/figterm-{session_id}.socket")),
        false =>  PathBuf::from(format!("/tmp/figterm-{session_id}.socket")),
    }
}
