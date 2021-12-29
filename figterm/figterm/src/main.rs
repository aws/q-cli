pub mod command_info;
pub mod fig_info;
pub mod history;
pub mod ipc;
pub mod logger;
pub mod proto;
pub mod pty;
pub mod term;
pub mod utils;

use std::{env, error::Error, ffi::CString, os::unix::prelude::AsRawFd, process::exit, vec};

use anyhow::{Context, Result};
use bytes::Bytes;
use dashmap::DashSet;
use fig_info::FigInfo;
use log::trace;
use nix::{
    libc::STDIN_FILENO,
    sys::termios::{tcgetattr, tcsetattr, SetArg},
    unistd::{execvp, getpid, isatty},
};

use alacritty_terminal::{
    event::{Event, EventListener},
    grid::Dimensions,
    index::{Column, Point},
    term::{cell::FigFlags, ShellState, SizeInfo},
    Term,
};
use proto::figterm::{figterm_message, intercept_command, FigtermMessage};
use pty::{fork_pty, PtyForkResult};
use term::get_winsize;
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    runtime, select,
    sync::mpsc::UnboundedSender,
};

use clap::Parser;

use crate::{
    ipc::{spawn_incoming_receiver, spawn_outgoing_sender},
    logger::init_logger,
    proto::{
        hooks::{
            hook_to_message, new_context, new_edit_buffer_hook, new_preexec_hook, new_prompt_hook,
        },
        local,
    },
    pty::{async_pty::AsyncPtyMaster, ioctl_tiocswinsz},
    term::{read_winsize, termios_to_raw},
};

const BUFFER_SIZE: usize = 1024;
const FIGTERM_VERSION: usize = 3;

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

fn shell_state_to_context(shell_state: &ShellState) -> local::ShellContext {
    #[cfg(target_os = "macos")]
    let terminal = utils::get_term_bundle();
    #[cfg(not(target_os = "macos"))]
    let term_bundle = None;

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
        terminal,
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
                let bytes = message.to_fig_pbuf().unwrap();

                self.sender.send(bytes).unwrap();
            }
            Event::PreExec => {
                let context = shell_state_to_context(shell_state);
                let hook = new_preexec_hook(Some(context));
                let message = hook_to_message(hook);
                let bytes = message.to_fig_pbuf().unwrap();

                self.sender.send(bytes).unwrap();
            }
        }
    }
}

struct EditBuffer {
    buffer: String,
    cursor: i64,
}

fn get_current_edit_buffer<T>(term: &Term<T>) -> Result<EditBuffer>
where
    T: EventListener,
{
    let start_point = Point::new(term.grid().cursor.point.line, Column(0));
    let end_point = Point::new(
        term.grid().cursor.point.line.min(term.bottommost_line()),
        term.last_column(),
    );

    let row = term.grid().iter_from_to(start_point, end_point);

    let mut whitespace_stack = String::new();
    let mut cursor_index = term.grid().cursor.point.column;
    let mut edit_buffer = String::new();

    for (_i, cell) in row.enumerate() {
        if !cell.fig_flags.contains(FigFlags::IN_PROMPT)
            && !cell.fig_flags.contains(FigFlags::IN_SUGGESTION)
        {
            if cell.c.is_whitespace() {
                whitespace_stack.push(cell.c);
            } else {
                if whitespace_stack.len() > 0 {
                    edit_buffer.push_str(&whitespace_stack);
                    whitespace_stack.clear();
                }
                edit_buffer.push(cell.c);
            }
        } else {
            cursor_index -= 1;
        }
    }

    if cursor_index > edit_buffer.len() {
        for _ in 0..(*cursor_index - edit_buffer.len()) {
            edit_buffer.push(' ');
        }
    }

    let cursor = (*cursor_index).try_into().unwrap_or(1) - 1;

    Ok(EditBuffer {
        buffer: edit_buffer,
        cursor,
    })
}

fn send_edit_buffer<T>(term: &Term<T>, sender: &UnboundedSender<Bytes>)
where
    T: EventListener,
{
    let edit_buffer = get_current_edit_buffer(term).unwrap();

    let context = shell_state_to_context(term.shell_state());
    let edit_buffer_hook =
        new_edit_buffer_hook(Some(context), edit_buffer.buffer, edit_buffer.cursor, 0);
    let message = hook_to_message(edit_buffer_hook);
    let bytes = message.to_fig_pbuf().unwrap();

    sender.send(bytes).unwrap();
}

async fn process_figterm_message(
    figterm_message: FigtermMessage,
    _term: &Term<EventSender>,
    pty_master: &mut AsyncPtyMaster,
    mut intercept_set: DashSet<char, fnv::FnvBuildHasher>,
) -> Result<()> {
    match figterm_message.command {
        Some(figterm_message::Command::InsertTextCommand(command)) => {
            pty_master.write(command.text.as_bytes()).await?;
        }
        Some(figterm_message::Command::InterceptCommand(command)) => {
            match command.intercept_command {
                Some(intercept_command::InterceptCommand::SetIntercept(set_intercept)) => {
                    trace!("Set intercept");
                    intercept_set.clear();
                    intercept_set.extend(
                        set_intercept
                            .chars
                            .iter()
                            .filter_map(|c| std::char::from_u32(*c)),
                    );
                }
                Some(intercept_command::InterceptCommand::ClearIntercept(_)) => {
                    trace!("Clear intercept");
                    intercept_set.clear();
                }
                Some(intercept_command::InterceptCommand::AddIntercept(set_intercept)) => {
                    trace!("{:?}", set_intercept.chars);
                    intercept_set.extend(
                        set_intercept
                            .chars
                            .iter()
                            .filter_map(|c| std::char::from_u32(*c)),
                    );
                }
                Some(intercept_command::InterceptCommand::RemoveIntercept(set_intercept)) => {
                    trace!("{:?}", set_intercept.chars);
                    for c in set_intercept.chars {
                        intercept_set.remove(&std::char::from_u32(c).unwrap());
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }

    Ok(())
}

fn launch_shell() -> Result<()> {
    let parent_shell = env::var("FIG_SHELL").expect("FIG_SHELL not set");
    let parent_shell_is_login = env::var("FIG_IS_LOGIN_SHELL").ok();
    let parent_shell_extra_args = env::var("FIG_SHELL_EXTRA_ARGS").ok();

    let mut args = vec![CString::new(&*parent_shell).unwrap()];

    if parent_shell_is_login.as_deref() == Some("1") {
        args.push(CString::new("--login").unwrap());
    }

    if let Some(extra_args) = parent_shell_extra_args {
        args.extend(
            extra_args
                .split_whitespace()
                .filter(|arg| arg != &"--login")
                .filter_map(|arg| CString::new(&*arg).ok()),
        );
    }

    env::set_var("FIG_TERM", "1");
    env::set_var("FIG_TERM_VERSION", format!("{}", FIGTERM_VERSION));
    if env::var_os("TMUX").is_some() {
        env::set_var("FIG_TERM_TMUX", "1");
    }

    // Clean up environment and launch shell.
    env::remove_var("FIG_SHELL");
    env::remove_var("FIG_IS_LOGIN_SHELL");
    env::remove_var("FIG_START_TEXT");
    env::remove_var("FIG_SHELL_EXTRA_ARGS");

    execvp(&*args[0], &args).expect("Failed to exec");
    unreachable!();
}

fn main() -> Result<(), Box<dyn Error>> {
    Args::parse();

    let fig_info = FigInfo::new();
    let _inital_command = std::env::var("FIG_START_TEXT").ok();

    // Get term data
    let termios = tcgetattr(STDIN_FILENO)?;
    let old_termios = termios.clone();

    let mut winsize = get_winsize(STDIN_FILENO)?;

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
                    let outgoing_tx = spawn_outgoing_sender().await?;

                    // Spawn thread to handle incoming data
                    let session_id = fig_info.term_session_id.unwrap_or("default".into());
                    let mut incomming_rx = spawn_incoming_receiver(&session_id).await?;

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

                    let intercept_set: DashSet<char, fnv::FnvBuildHasher> = DashSet::with_hasher(fnv::FnvBuildHasher::default());

                    // TODO: Write initial text to pty

                    'select_loop: loop {
                        select! {
                            biased;
                            res = stdin.read(&mut read_buffer) => {
                                if let Ok(size) = res {
                                    match std::str::from_utf8(&read_buffer[..size]) {
                                        Ok(s) => {
                                            for c in s.chars() {
                                                if !intercept_set.contains(&c) {
                                                    let mut utf8_buf = [0; 4];
                                                    master.write(c.encode_utf8(&mut utf8_buf).as_bytes()).await?;
                                                }
                                            }
                                        }
                                        Err(_) => {
                                            master.write(&read_buffer[..size]).await?;
                                        }
                                    }
                                }
                            }
                            res = master.read(&mut write_buffer) => {
                                match res {
                                    Ok(0) => {
                                        trace!("EOF from master");
                                        break 'select_loop;
                                    }
                                    Ok(size) => {
                                        trace!("Read {} bytes from master", size);

                                        for byte in &write_buffer[..size] {
                                            processor.advance(&mut term, *byte);
                                        }

                                        send_edit_buffer(&term, &outgoing_tx);

                                        stdout.write_all(&write_buffer[..size]).await?;
                                        stdout.flush().await?;
                                    }
                                    _ => {}
                                }
                            }
                            msg = incomming_rx.recv() => {
                                if let Some(buf) = msg {
                                    // TODO: convert this to protobufs!
                                    trace!("Received message from socket: {:?}", buf);

                                    process_figterm_message(buf, &term, &mut master, intercept_set.clone()).await?;
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
            launch_shell().expect("Failed to launch shell");
            unreachable!();
        }
    }
}
