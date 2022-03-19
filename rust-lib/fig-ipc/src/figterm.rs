use std::path::{Path, PathBuf};

pub fn get_figterm_socket_path(session_id: impl AsRef<str>) -> PathBuf {
    [
        Path::new("/mnt/c/fig"),
        Path::new(&["figterm-", session_id.as_ref(), ".socket"].concat()),
    ]
    .into_iter()
    .collect()
}
