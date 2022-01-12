mod arg_parser;
pub mod history;
pub mod ipc;
pub mod logger;
pub mod proto;
pub mod pty;
pub mod term;
pub mod utils;

use std::{env, error::Error, ffi::CString, os::unix::prelude::AsRawFd, process::exit, vec};

use anyhow::{anyhow, Context, Result};
use arg_parser::ArgParser;
use bytes::Bytes;
use dashmap::DashSet;
use flume::Sender;
use log::{debug, error, info, trace, warn};
use nix::{
    libc::STDIN_FILENO,
    sys::termios::{tcgetattr, tcsetattr, SetArg},
    unistd::{execvp, getpid, isatty},
};

use alacritty_terminal::term::CommandInfo;
use alacritty_terminal::{
    ansi::Processor,
    event::{Event, EventListener},
    term::{ShellState, SizeInfo},
    Term,
};
use proto::figterm::{figterm_message, intercept_command, FigtermMessage};
use pty::{fork_pty, PtyForkResult};
use term::get_winsize;
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    runtime, select,
    signal::unix::SignalKind,
};

use crate::{
    ipc::{spawn_incoming_receiver, spawn_outgoing_sender},
    logger::init_logger,
    proto::{
        hooks::{
            hook_to_message, new_context, new_edit_buffer_hook, new_preexec_hook, new_prompt_hook,
        },
        local, FigProtobufEncodable,
    },
    pty::{async_pty::AsyncPtyMaster, ioctl_tiocswinsz},
    term::{read_winsize, termios_to_raw},
};

const BUFFER_SIZE: usize = 1024;

struct EventSender {
    socket_sender: Sender<Bytes>,
    history_sender: Sender<CommandInfo>,
}

impl EventSender {
    fn new(socket_sender: Sender<Bytes>, history_sender: Sender<CommandInfo>) -> Self {
        Self {
            socket_sender,
            history_sender,
        }
    }
}

fn shell_state_to_context(shell_state: &ShellState) -> local::ShellContext {
    #[cfg(target_os = "macos")]
    let terminal = utils::get_term_bundle().map(|s| s.to_string());
    #[cfg(not(target_os = "macos"))]
    let terminal = None;

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
        debug!("{:?}", event);
        debug!("{:?}", shell_state);
        match event {
            Event::Prompt => {
                let context = shell_state_to_context(shell_state);
                let hook = new_prompt_hook(Some(context));
                let message = hook_to_message(hook);
                let bytes = message.encode_fig_protobuf().unwrap();

                self.socket_sender.send((*bytes).clone()).unwrap();
            }
            Event::PreExec => {
                let context = shell_state_to_context(shell_state);
                let hook = new_preexec_hook(Some(context));
                let message = hook_to_message(hook);
                let bytes = message.encode_fig_protobuf().unwrap();

                self.socket_sender.send((*bytes).clone()).unwrap();
            }
            Event::CommandInfo(command_info) => {
                self.history_sender.send(command_info.clone()).unwrap();
            }
        }
    }
}

fn can_send_edit_buffer<T>(term: &Term<T>) -> bool
where
    T: EventListener,
{
    let in_docker_ssh = term.shell_state().in_docker | term.shell_state().in_ssh;
    let shell_enabled =
        [Some("bash"), Some("zsh"), Some("fish")].contains(&term.shell_state().shell.as_deref());
    let prexec = term.shell_state().preexec;

    trace!(
        "in_docker_ssh: {}, shell_enabled: {}, prexec: {}",
        in_docker_ssh,
        shell_enabled,
        prexec
    );

    shell_enabled && !prexec
}

async fn send_edit_buffer<T>(term: &Term<T>, sender: &Sender<Bytes>) -> Result<()>
where
    T: EventListener,
{
    match term.get_current_buffer() {
        Some(edit_buffer) => {
            if let Some(cursor_idx) = edit_buffer.cursor_idx.map(|i| i.try_into().ok()).flatten() {
                log::info!("edit_buffer: {:?}", edit_buffer);

                let context = shell_state_to_context(term.shell_state());
                let edit_buffer_hook =
                    new_edit_buffer_hook(Some(context), edit_buffer.buffer, cursor_idx, 0);
                let message = hook_to_message(edit_buffer_hook);

                debug!("Sending: {:?}", message);

                let bytes = message.encode_fig_protobuf()?;

                sender.send_async((*bytes).clone()).await?;
            }
            Ok(())
        }
        None => Err(anyhow!("No edit buffer to send")),
    }
}

async fn process_figterm_message(
    figterm_message: FigtermMessage,
    _term: &Term<EventSender>,
    pty_master: &mut AsyncPtyMaster,
    mut intercept_set: DashSet<char, fnv::FnvBuildHasher>,
) -> Result<()> {
    match figterm_message.command {
        Some(figterm_message::Command::InsertTextCommand(command)) => {
            pty_master
                .write(command.to_term_string().as_bytes())
                .await?;
        }
        Some(figterm_message::Command::InterceptCommand(command)) => {
            match command.intercept_command {
                Some(intercept_command::InterceptCommand::SetIntercept(set_intercept)) => {
                    debug!("Set intercept");
                    intercept_set.clear();
                    intercept_set.extend(
                        set_intercept
                            .chars
                            .iter()
                            .filter_map(|c| std::char::from_u32(*c)),
                    );
                }
                Some(intercept_command::InterceptCommand::ClearIntercept(_)) => {
                    debug!("Clear intercept");
                    intercept_set.clear();
                }
                Some(intercept_command::InterceptCommand::AddIntercept(set_intercept)) => {
                    debug!("{:?}", set_intercept.chars);
                    intercept_set.extend(
                        set_intercept
                            .chars
                            .iter()
                            .filter_map(|c| std::char::from_u32(*c)),
                    );
                }
                Some(intercept_command::InterceptCommand::RemoveIntercept(set_intercept)) => {
                    debug!("{:?}", set_intercept.chars);
                    for c in set_intercept.chars {
                        intercept_set.remove(&std::char::from_u32(c).unwrap());
                    }
                }
                _ => {}
            }
        }
        Some(figterm_message::Command::SetBufferCommand(_command)) => {
            todo!();
        }
        _ => {}
    }

    Ok(())
}

fn launch_shell() -> Result<()> {
    let parent_shell = match env::var("FIG_SHELL").ok().filter(|s| !s.is_empty()) {
        Some(v) => v,
        None => match env::var("SHELL").ok().filter(|s| !s.is_empty()) {
            Some(shell) => shell,
            None => {
                anyhow::bail!("No FIG_SHELL or SHELL found");
            }
        },
    };

    let parent_shell_is_login = env::var("FIG_IS_LOGIN_SHELL")
        .ok()
        .filter(|s| !s.is_empty());
    let parent_shell_extra_args = env::var("FIG_SHELL_EXTRA_ARGS")
        .ok()
        .filter(|s| !s.is_empty());

    let mut args =
        vec![CString::new(&*parent_shell).expect("Failed to convert shell name to CString")];

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
    env::set_var("FIG_TERM_VERSION", env!("CARGO_PKG_VERSION"));
    if env::var_os("TMUX").is_some() {
        env::set_var("FIG_TERM_TMUX", "1");
    }

    // Clean up environment and launch shell.
    env::remove_var("FIG_SHELL");
    env::remove_var("FIG_IS_LOGIN_SHELL");
    env::remove_var("FIG_START_TEXT");
    env::remove_var("FIG_SHELL_EXTRA_ARGS");

    execvp(&*args[0], &args).unwrap();
    unreachable!()
}

fn figterm_main() -> Result<()> {
    let term_session_id = env::var("TERM_SESSION_ID")
        .with_context(|| "Failed to get TERM_SESSION_ID environment variable")?;

    let _fig_integration_version: Option<i32> = env::var("FIG_INTEGRATION_VERSION")
        .with_context(|| "Failed to get FIG_INTEGRATION_VERSION environment variable")?
        .parse()
        .ok();

    logger::stdio_debug_log("Checking stdin fd is a tty");

    // Check that stdin is a tty
    if !isatty(STDIN_FILENO).with_context(|| "Failed to check if stdin is a tty")? {
        anyhow::bail!("stdin is not a tty");
    }

    // Get term data
    let termios = tcgetattr(STDIN_FILENO).with_context(|| "Failed to get terminal attributes")?;
    let old_termios = termios.clone();

    let mut winsize = get_winsize(STDIN_FILENO).with_context(|| "Failed to get terminal size")?;

    logger::stdio_debug_log("Forking child shell process");

    // Fork pseudoterminal
    // SAFETY: forkpty is safe to call, but the child must not call any functions
    // that are not async-signal-safe.
    match fork_pty(&old_termios, &winsize)
        .context("fork_pty")
        .with_context(|| "Failed to fork pty")?
    {
        PtyForkResult::Parent(pt_details, pid) => {
            let runtime = runtime::Builder::new_multi_thread()
                .enable_all()
                .thread_name("figterm-thread")
                .build()?;

            init_logger(&pt_details.pty_name).with_context(|| "Failed to init logger")?;

            match runtime
                .block_on(async {

                    info!("Shell: {}", pid);
                    info!("Figterm: {}", getpid());

                    let history_sender = history::spawn_history_task().await;

                    let raw_termios = termios_to_raw(termios);
                    tcsetattr(STDIN_FILENO, SetArg::TCSAFLUSH, &raw_termios)?;

                    // Spawn thread to handle outgoing data to main Fig app
                    let outgoing_sender = spawn_outgoing_sender().await?;

                    // Spawn thread to handle incoming data
                    let incomming_reciever = spawn_incoming_receiver(&term_session_id).await?;

                    let mut stdin = io::stdin();
                    let mut stdout = io::stdout();
                    let mut master = AsyncPtyMaster::new(pt_details.pty_master)?;

                    let mut window_change_signal = tokio::signal::unix::signal(
                        SignalKind::window_change(),
                    )?;

                    let mut processor = Processor::new();
                    let size = SizeInfo::new(winsize.ws_row as usize, winsize.ws_col as usize);

                    let event_sender = EventSender::new(outgoing_sender.clone(), history_sender);

                    let mut term = alacritty_terminal::Term::new(size, event_sender, 1);

                    let mut read_buffer = [0u8; BUFFER_SIZE];
                    let mut write_buffer = [0u8; BUFFER_SIZE * 100];

                    let intercept_set: DashSet<char, fnv::FnvBuildHasher> = DashSet::with_hasher(fnv::FnvBuildHasher::default());

                    // TODO: Write initial text to pty

                    let mut first_time = true;

                    'select_loop: loop {

                        if term.shell_state().has_seen_prompt && first_time {
                            let initial_command = env::var("FIG_START_TEXT").ok().filter(|s| !s.is_empty());
                            if let Some(mut initial_command) = initial_command {
                                initial_command.push('\n');
                                if let Err(e) = master.write(initial_command.as_bytes()).await {
                                    error!("Failed to write initial command: {}", e);
                                }
                            }
                            first_time = false;
                        }

                        let select_result: Result<&'static str> = select! {
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
                                Ok("stdin")
                            }
                            _ = window_change_signal.recv() => {
                                unsafe { read_winsize(STDIN_FILENO, &mut winsize) }?;
                                unsafe { ioctl_tiocswinsz(master.as_raw_fd(), &winsize) }?;
                                term.resize(SizeInfo::new(winsize.ws_row as usize, winsize.ws_col as usize));
                                Ok("window_change")
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

                                        stdout.write_all(&write_buffer[..size]).await?;
                                        stdout.flush().await?;

                                        if can_send_edit_buffer(&term) {
                                            if let Err(e) = send_edit_buffer(&term, &outgoing_sender).await {
                                                warn!("Failed to send edit buffer: {}", e);
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                                Ok("master")
                            }
                            msg = incomming_reciever.recv_async() => {
                                if let Ok(buf) = msg {
                                    debug!("Received message from socket: {:?}", buf);
                                    process_figterm_message(buf, &term, &mut master, intercept_set.clone()).await?;
                                }
                                Ok("incomming_reciever")
                            }
                        };

                        if let Err(e) = select_result {
                            error!("Error in select loop: {}", e);
                            break 'select_loop;
                        }
                    }

                    tcsetattr(STDIN_FILENO, SetArg::TCSAFLUSH, &old_termios)?;

                    anyhow::Ok(())
                }) {
                    Ok(_) => {
                        info!("Exiting");
                        exit(0);
                    },
                    Err(e) => {
                        error!("Error in async runtime: {}", e);
                        exit(1);
                    },
                }
        }
        PtyForkResult::Child => {
            // DO NOT RUN ANY FUNCTIONS THAT ARE NOT ASYNC SIGNAL SAFE
            // https://man7.org/linux/man-pages/man7/signal-safety.7.html
            match launch_shell() {
                Err(e) => {
                    logger::stdio_debug_log(format!("{:?}", e));
                    Err(e)
                }
                Ok(_) => Ok(()),
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    ArgParser::new().parse();

    logger::stdio_debug_log(format!("FIG_LOG_LEVEL={}", logger::get_fig_log_level()));

    if let Err(e) = figterm_main() {
        logger::stdio_debug_log(format!("{}", e));

        // Fallback to shell if figterm fails
        launch_shell()?;
    }

    Ok(())
}
