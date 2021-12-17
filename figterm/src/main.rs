pub mod fig_info;
pub mod figterm;
pub mod history;
pub mod ipc;
pub mod local;
pub mod logger;
pub mod new_history;
pub mod proto;
pub mod pty;
pub mod screen;
pub mod term;
pub mod utils;

use std::{error::Error, ffi::CString, os::unix::prelude::AsRawFd, process::exit, time::Duration};

use alacritty_terminal::ansi::Processor;
use anyhow::{Context, Result};
use fig_info::FigInfo;
use nix::{
    libc::STDIN_FILENO,
    sys::termios::{tcgetattr, tcsetattr, SetArg},
    unistd::{execv, getpid, isatty},
};

use pty::{fork_pty, PtyForkResult};
use term::get_winsize;
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    runtime, select,
};

use clap::Parser;

use crate::{
    ipc::{connect_timeout, create_socket_listen, get_socket_path},
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
    match fork_pty(&old_termios, &winsize).context("fork_pty")? {
        PtyForkResult::Parent(pt_details, pid) => {
            let runtime = runtime::Builder::new_multi_thread().enable_all().build()?;
            runtime
                .block_on(async {
                    init_logger(&pt_details.pty_name).await?;

                    log::info!("Shell: {}", pid);
                    log::info!("Figterm: {}", getpid());

                    let raw_termios = termios_to_raw(termios);
                    tcsetattr(STDIN_FILENO, SetArg::TCSAFLUSH, &raw_termios)?;

                    // Spawn thread to handle outgoing data to main Fig app
                    let (outgoing_tx, mut outgoing_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(128);
                    tokio::spawn(async move {
                        let socket = get_socket_path();

                        while let Some(message) = outgoing_rx.recv().await {
                            let mut socket_conn =
                                connect_timeout(socket.clone(), Duration::from_secs(10))
                                    .await
                                    .unwrap();

                            socket_conn.write_all(&message).await.unwrap();
                            socket_conn.flush().await.unwrap();
                        }
                    });

                    // Spawn thread to handle incoming data
                    let socket_listener =
                        create_socket_listen(&fig_info.clone().term_session_id.unwrap()).await?;
                    let (incomming_tx, mut incomming_rx) = tokio::sync::mpsc::channel(128);
                    tokio::spawn(async move {
                        loop {
                            if let Ok((mut stream, _)) = socket_listener.accept().await {
                                let incomming_tx = incomming_tx.clone();
                                tokio::spawn(async move {
                                    let mut buf = Vec::new();
                                    while stream.read_to_end(&mut buf).await.is_ok() {
                                        incomming_tx.clone().send(buf.clone()).await.unwrap();
                                    }
                                });
                            }
                        }
                    });

                    let mut stdin = io::stdin();
                    let mut stdout = io::stdout();
                    let mut master = AsyncPtyMaster::new(pt_details.pty_master)?;

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

                    let mut parser = Processor::new();
                    let mut figterm = figterm::Figterm::new(outgoing_tx.clone(), fig_info.clone());

                    let mut read_buffer = [0u8; BUFFER_SIZE];
                    let mut write_buffer = [0u8; BUFFER_SIZE * 100];

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
                                        for byte in &write_buffer[..size] {
                                            parser.advance(&mut figterm, *byte);
                                        }

                                        stdout.write_all(&write_buffer[..size]).await?;
                                        stdout.flush().await?;
                                    }
                                    _ => {}
                                }
                            }
                            msg = incomming_rx.recv() => {
                                if let Some(buf) = msg {
                                    // TODO: convert this to protobufs!
                                    master.write(&buf).await?;
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
        PtyForkResult::Child => {
            // DO NOT RUN ANY FUNCTIONS THAT ARE NOT ASYNC SIGNAL SAFE
            // https://man7.org/linux/man-pages/man7/signal-safety.7.html
            execv(&shell, &[&shell]).unwrap();
            unreachable!();
        }
    }
}
