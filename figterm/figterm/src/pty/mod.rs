pub mod async_pty;

use anyhow::{Context, Result};
use nix::fcntl::{open, OFlag};
use nix::libc::{self, TIOCSCTTY};
use nix::pty::{grantpt, posix_openpt, ptsname, unlockpt, PtyMaster, Winsize};
use nix::sys::stat::Mode;
use nix::sys::termios::{tcsetattr, SetArg, Termios};
use nix::unistd::{close, dup2, fork, setsid, ForkResult, Pid};
use std::path::Path;

nix::ioctl_write_int_bad!(ioctl_tiocsctty, TIOCSCTTY);
nix::ioctl_write_ptr_bad!(ioctl_tiocswinsz, libc::TIOCSWINSZ, Winsize);

/// Psudoterminal Details
pub struct PtyDetails {
    /// Psudoterminal master fd wrapper
    pub pty_master: PtyMaster,
    /// Name of the psudoterminal
    pub pty_name: String,
}

/// Open a psudoterminal
fn open_pty() -> Result<PtyDetails> {
    // Open a new psudoterminal master
    let master_pty = posix_openpt(OFlag::O_RDWR)?;

    // Allow psudoterminal pair to be generated
    grantpt(&master_pty)?;
    unlockpt(&master_pty)?;

    // Get the name of the psudoterminal
    // SAFETY: This is done before any threads are spawned, thus it being
    // non thread safe is not an issue
    let pty_name = unsafe { ptsname(&master_pty) }?;

    Ok(PtyDetails {
        pty_master: master_pty,
        pty_name,
    })
}

/// Result of psudoterminal fork
pub enum PtyForkResult {
    /// Details of the psudoterminal and the [Pid] of the child
    Parent(PtyDetails, Pid),
    Child,
}

/// Forks the process, returns if the process is the Parent or Child
pub fn fork_pty(termios: &Termios, winsize: &Winsize) -> Result<PtyForkResult> {
    let pty_details = open_pty().context("Failed to open Psudoterminal")?;

    // SAFETY: Safe if if child does not run non async signal safe functions
    match unsafe { fork() }? {
        ForkResult::Parent { child } => Ok(PtyForkResult::Parent(pty_details, child)),
        ForkResult::Child => {
            // DO NOT RUN ANY FUNCTIONS THAT ARE NOT ASYNC SIGNAL SAFE
            // https://man7.org/linux/man-pages/man7/signal-safety.7.html

            setsid()?;

            let pty_fd = open(
                Path::new(&pty_details.pty_name),
                OFlag::O_RDWR,
                Mode::empty(),
            )?;

            #[cfg(target_os = "macos")]
            unsafe { ioctl_tiocsctty(pty_fd, 0) }?;

            tcsetattr(pty_fd, SetArg::TCSANOW, termios)?;
            unsafe { ioctl_tiocswinsz(pty_fd, winsize) }?;

            dup2(pty_fd, 0)?;
            dup2(pty_fd, 1)?;
            dup2(pty_fd, 2)?;

            if pty_fd > 2 {
                close(pty_fd)?;
            }

            Ok(PtyForkResult::Child)
        }
    }
}
