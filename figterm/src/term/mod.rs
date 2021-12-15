use std::os::unix::prelude::*;

use anyhow::Result;
use nix::{ioctl_read_bad, libc, pty::Winsize};

ioctl_read_bad!(read_winsize, libc::TIOCGWINSZ, Winsize);

/// Get the winsize of `fd`
pub fn get_winsize(fd: RawFd) -> Result<Winsize> {
    let mut winsize = Winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    unsafe { read_winsize(fd, &mut winsize) }?;

    Ok(winsize)
}
