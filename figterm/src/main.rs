pub mod fig;
pub mod history;
pub mod ipc;
pub mod local;
pub mod logger;
pub mod pty;
pub mod term;
pub mod utils;

use std::{error::Error, ffi::CString, os::unix::prelude::*, process::exit, time::Duration};

use anyhow::{Context, Result};
use fig::FigInfo;
use nix::{
    ioctl_write_ptr_bad,
    libc::{self, STDIN_FILENO},
    pty::Winsize,
    sys::termios::{
        cfmakeraw, tcgetattr, tcsetattr, ControlFlags, InputFlags, LocalFlags, OutputFlags, SetArg,
        Termios,
    },
    unistd::{self, execv, getpid, isatty},
};

use pty::{fork_pt, PtForkResult};
use term::get_winsize;
use tokio::{
    fs::File,
    io::{self, AsyncReadExt, AsyncWriteExt},
    runtime, select,
};

use clap::Parser;

use crate::{
    ipc::{connect_timeout, get_socket_path},
    logger::init_logger,
    pty::async_pty::AsyncPtyMaster,
};

const BUFFER_SIZE: usize = 1024;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(short, long)]
    version: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    Args::parse();

    let fig_info = FigInfo::new();
    let inital_command = std::env::var("FIG_START_TEXT").ok();

    // Get term data
    let termios = tcgetattr(STDIN_FILENO)?;
    let old_termios = termios.clone();

    let winsize = get_winsize(STDIN_FILENO)?;

    let shell = CString::new(std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into()))?;

    if !isatty(STDIN_FILENO)?
        || fig_info.fig_integration_version.is_none()
        || fig_info.term_session_id.is_none()
    {
        // Fallback
    }

    // Fork pseudoterminal
    // SAFETY: forkpty is safe to call, but the child must not call any functions
    // that are not async-signal-safe.
    match fork_pt(&old_termios, &winsize).context("fork_pty")? {
        PtForkResult::Parent(pt_details, pid) => {
            let runtime = runtime::Builder::new_multi_thread().enable_all().build()?;
            runtime
                .block_on(async {
                    init_logger(&pt_details.slave_name).await?;

                    log::info!("Shell: {}", pid);
                    log::info!("Figterm: {}", getpid());

                    let raw_termios = to_raw(termios);
                    tcsetattr(STDIN_FILENO, SetArg::TCSAFLUSH, &raw_termios)?;

                    // Spawn async green thread to handle sending data to Mac app
                    let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<u8>>(128);
                    tokio::spawn(async move {
                        let socket = get_socket_path();
                        let mut socket_conn = connect_timeout(socket, Duration::from_secs(10))
                            .await
                            .unwrap();

                        while let Some(message) = rx.recv().await {
                            socket_conn.write_all(&message).await.unwrap();
                            socket_conn.flush().await.unwrap();
                        }
                    });

                    let mut stdin = io::stdin();
                    let mut stdout = io::stdout();
                    let mut master = AsyncPtyMaster::new(pt_details.master_pty)?;

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

                    let mut read_buffer = [0u8; BUFFER_SIZE];
                    let mut write_buffer = [0u8; BUFFER_SIZE];

                    'select_loop: loop {
                        select! {
                            biased;
                            res = stdin.read(&mut read_buffer) => {
                                if let Ok(size) = res {
                                    master.write(&read_buffer[..size]).await?;
                                }
                            }
                            res = master.read(&mut write_buffer) => {
                                match res {
                                    Ok(0) => break 'select_loop,
                                    Ok(size) => {
                                        stdout.write_all(&write_buffer[..size]).await?;
                                        stdout.flush().await?;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }

                    tcsetattr(STDIN_FILENO, SetArg::TCSAFLUSH, &old_termios)?;

                    Ok::<(), Box<dyn Error>>(())
                })
                .unwrap();

            exit(0);
        }
        PtForkResult::Child => {
            // DO NOT RUN ANY FUNCTIONS THAT ARE NOT ASYNC SIGNAL SAFE
            // https://man7.org/linux/man-pages/man7/signal-safety.7.html
            execv(&shell, &[&shell]).unwrap();
            unreachable!();
        }
    }
}

fn to_raw(mut termios: Termios) -> Termios {
    // Turn off echo, canonical mode extended input processing & signal chars.
    termios
        .local_flags
        .remove(LocalFlags::ECHO | LocalFlags::ICANON | LocalFlags::IEXTEN | LocalFlags::ISIG);

    // Turn off SIGINT on BREAK, CR-to-NL, input parity check, strip 8th bit on input,
    // and output control flow.
    termios.input_flags.remove(
        InputFlags::BRKINT
            | InputFlags::ICRNL
            | InputFlags::IGNBRK
            | InputFlags::INPCK
            | InputFlags::ISTRIP
            | InputFlags::IXON,
    );

    // Clear size bits, parity checking off.
    termios
        .control_flags
        .remove(ControlFlags::CSIZE | ControlFlags::PARENB);

    // 8 bits/char
    termios.control_flags.insert(ControlFlags::CS8);

    // Output processing off.
    termios.output_flags.remove(OutputFlags::OPOST);

    // Set case b, 1 byte at a time, no timer.
    termios.control_chars[libc::VMIN] = 1;
    termios.control_chars[libc::VTIME] = 0;

    termios
}
