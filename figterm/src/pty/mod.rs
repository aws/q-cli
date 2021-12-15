pub mod async_pty;

use anyhow::Result;
use nix::fcntl::{open, OFlag};
use nix::libc::{self, TIOCSCTTY};
use nix::pty::{grantpt, posix_openpt, ptsname, unlockpt, PtyMaster, Winsize};
use nix::sys::stat::Mode;
use nix::sys::termios::{tcsetattr, SetArg, Termios};
use nix::unistd::{close, dup2, fork, setsid, ForkResult, Pid};
use std::path::Path;

nix::ioctl_write_int_bad!(ioctl_tiocsctty, TIOCSCTTY);
nix::ioctl_write_ptr_bad!(ioctl_tiocswinsz, libc::TIOCSWINSZ, Winsize);

pub struct PtDetails {
    pub master_pty: PtyMaster,
    pub slave_name: String,
}

fn open_pt() -> Result<PtDetails> {
    // Open a new PTY master
    let master_pty = posix_openpt(OFlag::O_RDWR).unwrap();

    // Allow a slave to be generated for it
    grantpt(&master_pty).unwrap();
    unlockpt(&master_pty).unwrap();

    // Get the name of the slave
    let slave_name = unsafe { ptsname(&master_pty) }.unwrap();

    Ok(PtDetails {
        master_pty,
        slave_name,
    })
}
pub enum PtForkResult {
    Parent(PtDetails, Pid),
    Child,
}

pub fn fork_pt(termios: &Termios, winsize: &Winsize) -> Result<PtForkResult> {
    let pt_details = open_pt().unwrap();

    match unsafe { fork() }.unwrap() {
        ForkResult::Parent { child } => {
            // set_nonblocking(pt_details.master_pty.as_raw_fd()).unwrap();
            Ok(PtForkResult::Parent(pt_details, child))
        }
        ForkResult::Child => {
            setsid().unwrap();

            let slave_fd = open(
                Path::new(&pt_details.slave_name),
                OFlag::O_RDWR,
                Mode::empty(),
            )
            .unwrap();

            #[cfg(any(target_os = "macos"))]
            unsafe { ioctl_tiocsctty(slave_fd, 0) }.unwrap();

            tcsetattr(slave_fd, SetArg::TCSANOW, termios).unwrap();
            unsafe { ioctl_tiocswinsz(slave_fd, winsize) }.unwrap();

            dup2(slave_fd, 0).unwrap();
            dup2(slave_fd, 1).unwrap();
            dup2(slave_fd, 2).unwrap();

            if slave_fd > 2 {
                close(slave_fd).unwrap();
            }

            Ok(PtForkResult::Child)
        }
    }
}
