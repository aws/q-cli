pub mod async_pty;

use anyhow::Result;
use nix::fcntl::{open, OFlag};
use nix::libc::TIOCSCTTY;
use nix::pty::{grantpt, posix_openpt, ptsname, unlockpt, PtyMaster};
use nix::sys::stat::Mode;
use nix::unistd::{close, dup2, fork, setsid, ForkResult, Pid};
use std::path::Path;

nix::ioctl_write_int_bad!(ioctl_tiocsctty, TIOCSCTTY);

pub struct PtDetails {
    pub pty_master: PtyMaster,
    pub slave_name: String,
}

fn open_pt() -> Result<PtDetails> {
    // Open a new PTY master
    let pty_master = posix_openpt(OFlag::O_RDWR)?;

    // Allow a slave to be generated for it
    grantpt(&pty_master)?;
    unlockpt(&pty_master)?;

    // Get the name of the slave
    let slave_name = unsafe { ptsname(&pty_master) }?;

    Ok(PtDetails {
        pty_master,
        slave_name,
    })
}
pub enum PtForkResult {
    Parent(PtDetails, Pid),
    Child,
}

pub fn fork_pt() -> Result<PtForkResult> {
    let pt_details = open_pt()?;

    match unsafe { fork() }? {
        ForkResult::Parent { child } => Ok(PtForkResult::Parent(pt_details, child)),
        ForkResult::Child => {
            setsid()?;

            let slave_fd = open(
                Path::new(&pt_details.slave_name),
                OFlag::O_RDWR,
                Mode::empty(),
            )?;

            unsafe { ioctl_tiocsctty(slave_fd, 0) }?;

            dup2(slave_fd, 0)?;
            dup2(slave_fd, 1)?;
            dup2(slave_fd, 2)?;

            if slave_fd > 2 {
                close(slave_fd)?;
            }

            Ok(PtForkResult::Child)
        }
    }
}
