use std::fs::File;
use std::os::unix::prelude::AsRawFd;
use std::path::PathBuf;

use anyhow::Result;
use fig_util::directories;
use nix::fcntl::{
    flock,
    FlockArg,
};
use procfs::net::UnixState;

pub fn cleanup() -> Result<()> {
    if fig_util::in_ssh() {
        let lockfile = File::create(directories::fig_ephemeral_dir()?.join("cleanup.lock"))?;
        flock(lockfile.as_raw_fd(), FlockArg::LockExclusive)?;

        let secure_path = directories::secure_socket_path()?;
        if !is_held(&secure_path)? {
            std::fs::remove_file(secure_path)?;
        }
    }

    Ok(())
}

/// checks if the socket at the target path is connected to by any process (including itself)
fn is_held(check_path: &PathBuf) -> Result<bool> {
    for entry in procfs::net::unix()? {
        if let Some(unix_path) = entry.path {
            if &unix_path == check_path && entry.ref_count > 0 && entry.state != UnixState::DISCONNECTING {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
