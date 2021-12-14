pub mod ipc;
pub mod local;
pub mod logger;
pub mod pty;

use std::{error::Error, ffi::CString, os::unix::prelude::*};

use anyhow::Result;
use nix::{
    ioctl_read_bad, ioctl_write_ptr_bad, libc,
    pty::Winsize,
    sys::termios::{cfmakeraw, tcgetattr, tcsetattr, SetArg},
    unistd::execv,
};

use pty::{fork_pt, PtForkResult};
use tokio::{
    fs::File,
    io::{self, AsyncReadExt, AsyncWriteExt},
    runtime, select,
};

use clap::Parser;

ioctl_read_bad!(read_winsize, libc::TIOCGWINSZ, Winsize);
ioctl_write_ptr_bad!(tiocswinsz, libc::TIOCSWINSZ, Winsize);

const BUFFER_SIZE: usize = 1024;
const FIGTERM_VERSION: &'static str = "4";

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(short, long)]
    version: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    Args::parse();

    let stdin = io::stdin();

    // Get term data
    let termios = tcgetattr(stdin.as_raw_fd())?;
    let mut winsize = Winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    unsafe { read_winsize(stdin.as_raw_fd(), &mut winsize) }?;

    let shell = CString::new(std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into()))?;

    // Fork pseudoterminal
    // SAFETY: forkpty is safe to call, but the child must not call any functions
    // that are not async-signal-safe.
    let fork = fork_pt()?;

    match fork {
        PtForkResult::Parent(pt_details, _) => {
            let runtime = runtime::Builder::new_current_thread().build()?;

            runtime.block_on(async {
                // let old_termios = tty_set_raw(stdin.as_raw_fd()).unwrap();
                let mut old_termios = termios.clone();
                cfmakeraw(&mut old_termios);
                tcsetattr(stdin.as_raw_fd(), SetArg::TCSAFLUSH, &old_termios)?;

                // SAFETY: We are provided the file discriptor of the pseudoterminal from the fork
                // which we know will provide a valid file descriptor.
                let mut master = io::BufReader::new(unsafe {
                    File::from_raw_fd(pt_details.master_fd.as_raw_fd())
                });

                let mut stdin = io::stdin();
                let mut stdout = io::stdout();

                // let mut signals = Signals::new(&[SIGWINCH]).unwrap();
                // let stdin_fd = stdin.as_raw_fd();

                // TODO: Move signal check into main loop via async
                // std::thread::spawn(move || {
                //     let mut winsize = winsize.clone();

                //     for signal in signals.forever() {
                //         match signal {
                //             SIGWINCH => {
                //                 unsafe { read_winsize(stdin_fd, &mut winsize) }.unwrap();
                //                 unsafe { tiocswinsz(fork.master, &winsize) }.unwrap();
                //             }
                //             _ => {}
                //         }
                //     }
                // });

                'select_loop: loop {
                    let mut read_buffer = [0u8; BUFFER_SIZE];
                    let mut write_buffer = [0u8; BUFFER_SIZE];


                    select! {
                        res = stdin.read(&mut read_buffer) => {
                            if let Ok(size) = res {
                                nix::unistd::write(pt_details.master_fd.as_raw_fd(), &read_buffer[..size])?;
                            }
                        }
                        res = master.read(&mut write_buffer) => {
                            if let Ok(size) = res {
                                stdout.write_all(&write_buffer[..size]).await?;
                                stdout.flush().await?;

                                if size == 0 {
                                    break 'select_loop;
                                }
                            }
                        }
                    }
                }

                tcsetattr(stdin.as_raw_fd(), SetArg::TCSAFLUSH, &old_termios)?;

                Ok::<(), Box<dyn Error>>(())
            })?;
        }
        PtForkResult::Child => {
            // DO NOT RUN ANY FUNCTIONS THAT ARE NOT ASYNC SIGNAL SAFE
            // https://man7.org/linux/man-pages/man7/signal-safety.7.html
            execv(&shell, &[&shell]).unwrap();
            unreachable!();
        }
    }

    Ok(())
}
