use std::{
    io::{Read, Write},
    os::unix::prelude::{AsRawFd, RawFd},
};

use nix::{
    fcntl::{self, FcntlArg, OFlag},
    pty::PtyMaster,
};
use tokio::io::{self, unix::AsyncFd};

use anyhow::{Context, Result};

/// An async wrapper over `PtyMaster`
pub struct AsyncPtyMaster(AsyncFd<PtyMaster>);

impl AsyncPtyMaster {
    pub fn new(pty_master: PtyMaster) -> Result<Self> {
        set_nonblocking(pty_master.as_raw_fd()).with_context(|| "Failed to set nonblocking")?;
        Ok(Self(
            AsyncFd::new(pty_master).with_context(|| "Failed to create AsyncFd")?,
        ))
    }

    pub async fn read(&mut self, buff: &mut [u8]) -> io::Result<usize> {
        loop {
            let mut guard = self.0.readable_mut().await?;

            match guard.try_io(|inner| inner.get_mut().read(buff)) {
                Ok(result) => return result,
                Err(_would_block) => continue,
            }
        }
    }

    pub async fn write(&mut self, buff: &[u8]) -> io::Result<usize> {
        loop {
            let mut guard = self.0.writable_mut().await?;

            match guard.try_io(|inner| inner.get_mut().write(buff)) {
                Ok(result) => return result,
                Err(_would_block) => continue,
            }
        }
    }
}

impl AsRawFd for AsyncPtyMaster {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

/// Set `fd` into non-blocking mode using O_NONBLOCKING
fn set_nonblocking(fd: RawFd) -> Result<()> {
    let old_oflag_c_int = fcntl::fcntl(fd, FcntlArg::F_GETFL)
        .with_context(|| format!("Failed to get flags for fd {:?}", fd))?;

    let old_oflag = OFlag::from_bits_truncate(old_oflag_c_int);
    // .with_context(|| format!("Failed to convert c_int {:?} to OFlag", old_oflag_c_int))?;

    fcntl::fcntl(fd, FcntlArg::F_SETFL(old_oflag | OFlag::O_NONBLOCK))
        .with_context(|| format!("Failed to set O_NONBLOCK for fd {:?}", fd))?;

    Ok(())
}
