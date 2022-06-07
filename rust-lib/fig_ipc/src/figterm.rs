use std::path::PathBuf;

use wsl::is_wsl;

pub fn get_figterm_socket_path(session_id: impl AsRef<str>) -> PathBuf {
    match is_wsl() {
        true => PathBuf::from(format!("/mnt/c/fig/figterm-{}.socket", session_id.as_ref())),
        false => PathBuf::from(format!("/tmp/figterm-{}.socket", session_id.as_ref())),
    }
}
