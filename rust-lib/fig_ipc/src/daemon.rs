use std::path::{
    Path,
    PathBuf,
};

/// Get path to "$TMPDIR/fig/daemon.sock"
pub fn get_daemon_socket_path() -> PathBuf {
    [
        std::env::temp_dir().as_path(),
        Path::new("fig"),
        Path::new("daemon.sock"),
    ]
    .into_iter()
    .collect()
}
