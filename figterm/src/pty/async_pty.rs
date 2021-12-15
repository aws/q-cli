use std::{
    io::{Read, Write},
    os::unix::prelude::{AsRawFd, RawFd},
};

use nix::{
    fcntl::{self, FcntlArg, OFlag},
    pty::PtyMaster,
};
use tokio::io::{self, unix::AsyncFd};

use anyhow::Result;

/// An async wrapper over `PtyMaster`
pub struct AsyncPtyMaster(AsyncFd<PtyMaster>);

impl AsyncPtyMaster {
    pub fn new(pty_master: PtyMaster) -> io::Result<Self> {
        set_nonblocking(pty_master.as_raw_fd()).unwrap();
        Ok(Self(AsyncFd::new(pty_master)?))
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
    let old_flag = OFlag::from_bits(fcntl::fcntl(fd, FcntlArg::F_GETFL).unwrap()).unwrap();

    fcntl::fcntl(fd, FcntlArg::F_SETFL(old_flag | OFlag::O_NONBLOCK)).unwrap();

    Ok(())
}
