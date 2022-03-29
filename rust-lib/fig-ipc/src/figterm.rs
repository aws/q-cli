use std::path::{Path, PathBuf};
use wsl::is_wsl;

pub fn get_figterm_socket_path(session_id: impl AsRef<str>) -> PathBuf {
    // TODO: Good WSL socket path?
    [
        Path::new(if is_wsl() { "/mnt/c/fig" } else { "/tmp" }),
        Path::new(&["figterm-", session_id.as_ref(), ".socket"].concat()),
    ]
    .into_iter()
    .collect()
}
