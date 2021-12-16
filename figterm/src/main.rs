pub mod fig;
pub mod history;
pub mod ipc;
pub mod local;
pub mod logger;
pub mod proto;
pub mod pty;
pub mod term;
pub mod utils;
pub mod new_history;

use std::{error::Error, ffi::CString, os::unix::prelude::AsRawFd, process::exit, time::Duration};

use anyhow::{Context, Result};
use fig::FigInfo;
use nix::{
    libc::STDIN_FILENO,
    sys::termios::{
        tcgetattr, tcsetattr, SetArg,
    },
    unistd::{execv, getpid, isatty},
};

use pty::{fork_pt, PtForkResult};
use term::get_winsize;
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    runtime, select,
};

use clap::Parser;

use crate::{
    ipc::{connect_timeout, get_socket_path},
    logger::init_logger,
    pty::{async_pty::AsyncPtyMaster, ioctl_tiocswinsz},
    term::{read_winsize, termios_to_raw},
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
    let _inital_command = std::env::var("FIG_START_TEXT").ok();

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

                    let raw_termios = termios_to_raw(termios);
                    tcsetattr(STDIN_FILENO, SetArg::TCSAFLUSH, &raw_termios)?;

                    // Spawn async green thread to handle sending data to Mac app
                    let (_tx, mut rx) = tokio::sync::mpsc::channel::<Vec<u8>>(128);
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

                    // TODO: make this more safe!
                    let master_fd = master.as_raw_fd();

                    let mut window_change_signal = tokio::signal::unix::signal(
                        tokio::signal::unix::SignalKind::window_change(),
                    )?;

                    tokio::spawn(async move {
                        let mut winsize = winsize;

                        loop {
                            window_change_signal.recv().await;
                            unsafe { read_winsize(STDIN_FILENO, &mut winsize) }.unwrap();
                            unsafe { ioctl_tiocswinsz(master_fd, &winsize) }.unwrap();
                        }
                    });

                    let _history = new_history::History::load()?;

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
