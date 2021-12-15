use std::io::{Read, Write};

use nix::pty::PtyMaster;
use tokio::io::{self, unix::AsyncFd};

pub struct AsyncPtyMaster(AsyncFd<PtyMaster>);

impl AsyncPtyMaster {
    pub fn new(pty_master: PtyMaster) -> io::Result<Self> {
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

    pub fn write(&mut self, out: &[u8]) -> usize {
        self.0.get_mut().write(out).unwrap()
    }
}
