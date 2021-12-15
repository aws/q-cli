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

pub struct AsyncPtyMaster(AsyncFd<PtyMaster>);

impl AsyncPtyMaster {
    pub fn new(pty_master: PtyMaster) -> io::Result<Self> {
        set_nonblocking(pty_master.as_raw_fd()).unwrap();
        Ok(Self(AsyncFd::new(pty_master)?))
    }

    pub async fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        loop {
            let mut guard = self.0.readable_mut().await?;

            match guard.try_io(|inner| inner.get_mut().read(out)) {
                Ok(result) => return result,
                Err(_would_block) => continue,
            }
        }
    }

    pub async fn write(&mut self, out: &[u8]) -> io::Result<usize> {
        loop {
            let mut guard = self.0.writable_mut().await?;

            match guard.try_io(|inner| inner.get_mut().write(out)) {
                Ok(result) => return result,
                Err(_would_block) => continue,
            }
        }
    }
}

pub fn set_nonblocking(fd: RawFd) -> Result<()> {
    let old_flag = OFlag::from_bits(fcntl::fcntl(fd, FcntlArg::F_GETFL).unwrap()).unwrap();

    fcntl::fcntl(fd, FcntlArg::F_SETFL(old_flag | OFlag::O_NONBLOCK)).unwrap();

    Ok(())
}
