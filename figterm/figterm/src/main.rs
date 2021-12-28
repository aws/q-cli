pub mod command_info;
pub mod fig_info;
pub mod ipc;
pub mod local;
pub mod logger;
pub mod new_history;
pub mod proto;
pub mod pty;
pub mod term;
pub mod utils;

use std::{
    error::Error, ffi::CString, iter::repeat, os::unix::prelude::AsRawFd, process::exit,
    time::Duration,
};

use anyhow::{Context, Result};
use bytes::Bytes;
use fig_info::FigInfo;
use log::trace;
use nix::{
    libc::STDIN_FILENO,
    sys::termios::{tcgetattr, tcsetattr, SetArg},
    unistd::{execv, getpid, isatty},
};

use alacritty_terminal::{
    event::{Event, EventListener},
    grid::Dimensions,
    term::{cell::FigFlags, ShellState, SizeInfo},
    Term,
};
use pty::{fork_pty, PtyForkResult};
use term::get_winsize;
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    runtime, select,
    sync::mpsc::UnboundedSender,
};

use clap::Parser;

use crate::{
    ipc::{connect_timeout, create_socket_listen, get_socket_path},
    logger::init_logger,
    proto::hooks::{
        hook_to_message, new_context, new_edit_buffer_hook, new_preexec_hook, new_prompt_hook,
    },
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

struct EventSender {
    sender: UnboundedSender<Bytes>,
}

impl EventSender {
    fn new(sender: UnboundedSender<Bytes>) -> Self {
        Self { sender }
    }
}

fn shell_state_to_context(shell_state: &ShellState) -> crate::proto::ShellContext {
    new_context(
        shell_state.pid,
        shell_state.tty.clone(),
        shell_state.shell.clone(),
        shell_state
            .current_working_directory
            .clone()
            .map(|cwd| cwd.display().to_string()),
        shell_state.session_id.clone(),
        None,
        utils::get_term_bundle(),
        shell_state.hostname.clone(),
    )
}

impl EventListener for EventSender {
    fn send_event(&self, event: Event, shell_state: &ShellState) {
        trace!("{:?}", event);
        trace!("{:?}", shell_state);
        match event {
            Event::Prompt => {
                let context = shell_state_to_context(shell_state);
                let hook = new_prompt_hook(Some(context));
                let message = hook_to_message(hook);
                let bytes = message.to_fig_pbuf();

                self.sender.send(bytes).unwrap();
            }
            Event::PreExec => {
                let context = shell_state_to_context(shell_state);
                let hook = new_preexec_hook(Some(context));
                let message = hook_to_message(hook);
                let bytes = message.to_fig_pbuf();

                self.sender.send(bytes).unwrap();
            }
        }
    }
}

fn edit_buffer<T>(term: &Term<T>, sender: &UnboundedSender<Bytes>)
where
    T: EventListener,
{
    let mut line_start = term.grid().cursor.point;
    line_start.column.0 = 0;
    let mut line_end = term.grid().cursor.point;
    line_end.column.0 = term.columns();
    let row = term.grid().iter_from_to(line_start, line_end);

    let mut last_cell = 100000;
    let mut cursor_index = term.grid().cursor.point.column;
    let mut line = String::new();
    for (i, cell) in row.enumerate() {
        if !cell.fig_flags.contains(FigFlags::IN_PROMPT)
            && !cell.fig_flags.contains(FigFlags::IN_SUGGESTION)
        {
            if cell.c != ' ' {
                if last_cell + 1 < i {
                    for _ in 0..i - last_cell - 1 {
                        line.push(' ');
                    }
                }
                line.push(cell.c);
                last_cell = i;
            } else {
            }
        } else {
            cursor_index -= 1;
        }
    }

    let v_len = line.chars().count() as i64;
    // if v_len > 0 {
    let context = shell_state_to_context(term.shell_state());
    let edit_buffer_hook = new_edit_buffer_hook(Some(context), line, v_len, 0);
    let message = hook_to_message(edit_buffer_hook);
    let bytes = message.to_fig_pbuf();
    sender.send(bytes).unwrap();
    // }
}

fn main() -> Result<(), Box<dyn Error>> {
    Args::parse();

    let fig_info = FigInfo::new();
    let _inital_command = std::env::var("FIG_START_TEXT").ok();

    // Get term data
    let termios = tcgetattr(STDIN_FILENO)?;
    let old_termios = termios.clone();

    let mut winsize = get_winsize(STDIN_FILENO)?;

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
                    let (outgoing_tx, mut outgoing_rx) =
                        tokio::sync::mpsc::unbounded_channel::<Bytes>();
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
                        create_socket_listen(&fig_info.clone().term_session_id.unwrap_or("default".into())).await?;
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

                    let mut window_change_signal = tokio::signal::unix::signal(
                        tokio::signal::unix::SignalKind::window_change(),
                    )?;

                    let mut processor = alacritty_terminal::ansi::Processor::new();
                    let size = SizeInfo::new(winsize.ws_row as usize, winsize.ws_col as usize);

                    let event_sender = EventSender::new(outgoing_tx.clone());

                    let mut term = alacritty_terminal::Term::new(size, event_sender, 1);

                    let mut read_buffer = [0u8; BUFFER_SIZE];
                    let mut write_buffer = [0u8; BUFFER_SIZE * 100];

                    let mut line_start = term.grid().cursor.point;
                    line_start.column.0 = 0;
                    let mut line_end = term.grid().cursor.point;
                    line_end.column.0 = term.columns();
                    let row = term.grid().iter_from_to(line_start, line_end);

                    let mut last_cell = 0;
                    let mut cursor_index = term.grid().cursor.point.column;
                    let mut line = String::new();
                    for (i, cell) in row.enumerate() {
                        if !cell.fig_flags.contains(FigFlags::IN_PROMPT) && !cell.fig_flags.contains(FigFlags::IN_SUGGESTION) {
                            if cell.c != ' ' {
                                if last_cell < i {
                                    line.push_str(repeat(' ').take(i - last_cell).collect::<String>().as_str());
                                }
                                line.push(cell.c);
                                last_cell = i;
                            } else {

                            }
                        } else {
                            cursor_index -= 1;
                        }
                    }

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
                                            processor.advance(&mut term, *byte);

                                                // log::info!("{}", v);

                                            // log::info!("{:?}", screen.renderable_content().cursor.point);
                                        }

                                        edit_buffer(&term, &outgoing_tx);

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
                            _ = window_change_signal.recv() => {
                                unsafe { read_winsize(STDIN_FILENO, &mut winsize) }.unwrap();
                                unsafe { ioctl_tiocswinsz(master.as_raw_fd(), &winsize) }.unwrap();
                                term.resize(SizeInfo::new(winsize.ws_row as usize, winsize.ws_col as usize));
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
