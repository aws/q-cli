pub mod cli;
pub mod history;
pub mod ipc;
pub mod logger;
pub mod pty;
pub mod term;
pub mod utils;

use crate::{
    ipc::{remove_socket, spawn_incoming_receiver, spawn_outgoing_sender},
    logger::init_logger,
    pty::{async_pty::AsyncPtyMaster, fork_pty, ioctl_tiocswinsz, PtyForkResult},
    term::get_winsize,
    term::{read_winsize, termios_to_raw},
};

use alacritty_terminal::{
    ansi::Processor,
    event::{Event, EventListener},
    term::{CommandInfo, ShellState, SizeInfo},
    Term,
};
use anyhow::{anyhow, Context, Result};
use clap::StructOpt;
use cli::Cli;
use dashmap::DashSet;
use fig_auth::get_email;
use fig_directories::fig_dir;
use fig_proto::{
    figterm::{figterm_message, intercept_command, FigtermMessage},
    hooks::{
        hook_to_message, new_context, new_edit_buffer_hook, new_preexec_hook, new_prompt_hook,
    },
    local::{self, LocalMessage},
};
use flume::Sender;
use nix::{
    libc::STDIN_FILENO,
    sys::termios::{tcgetattr, tcsetattr, SetArg},
    unistd::{execvp, getpid, isatty},
};
use once_cell::sync::Lazy;
use parking_lot::lock_api::RawMutex as RawMutexTrait;
use parking_lot::{Mutex, RawMutex};
use sentry::integrations::anyhow::capture_anyhow;
use std::time::{Duration, SystemTime};
use std::{
    env, error::Error, ffi::CString, os::unix::prelude::AsRawFd, process::exit, str::FromStr, vec,
};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    runtime, select,
    signal::unix::SignalKind,
};
use tracing::{debug, error, info, level_filters::LevelFilter, trace, warn};

const BUFFER_SIZE: usize = 4096;

struct EventSender {
    socket_sender: Sender<LocalMessage>,
    history_sender: Sender<CommandInfo>,
}

impl EventSender {
    fn new(socket_sender: Sender<LocalMessage>, history_sender: Sender<CommandInfo>) -> Self {
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
    let integration_version = std::env::var("FIG_INTEGRATION_VERSION")
        .map(|s| s.parse().ok())
        .ok()
        .flatten()
        .unwrap_or(7);

    let mut context = new_context(
        shell_state.local_context.pid,
        shell_state.local_context.tty.clone(),
        shell_state.local_context.shell.clone(),
        shell_state
            .local_context
            .current_working_directory
            .clone()
            .map(|cwd| cwd.display().to_string()),
        shell_state.local_context.session_id.clone(),
        Some(integration_version),
        terminal.clone(),
        shell_state.local_context.hostname.clone(),
    );

    if shell_state.in_ssh || shell_state.in_docker {
        let remote_context = new_context(
            shell_state.remote_context.pid,
            shell_state.remote_context.tty.clone(),
            shell_state.remote_context.shell.clone(),
            shell_state
                .remote_context
                .current_working_directory
                .clone()
                .map(|cwd| cwd.display().to_string()),
            shell_state.remote_context.session_id.clone(),
            Some(integration_version),
            terminal,
            shell_state.remote_context.hostname.clone(),
        );
        context.remote_context = Some(Box::new(remote_context));
    }
    context
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
                if let Err(err) = self.socket_sender.send(message) {
                    error!("Sender error: {:?}", err);
                }
            }
            Event::PreExec => {
                let context = shell_state_to_context(shell_state);
                let hook = new_preexec_hook(Some(context));
                let message = hook_to_message(hook);
                if let Err(err) = self.socket_sender.send(message) {
                    error!("Sender error: {:?}", err);
                }
            }
            Event::CommandInfo(command_info) => {
                if let Err(err) = self.history_sender.send(command_info.clone()) {
                    error!("Sender error: {:?}", err);
                }
            }
        }
    }

    fn log_level_event(&self, level: Option<String>) {
        logger::set_log_level(
            level
                .and_then(|level| LevelFilter::from_str(&level).ok())
                .unwrap_or(LevelFilter::INFO),
        );
    }
}

static INSERTION_LOCKED_AT: Mutex<Option<SystemTime>> = Mutex::const_new(RawMutex::INIT, None);
static EXPECTED_BUFFER: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new("".to_string()));

fn can_send_edit_buffer<T>(term: &Term<T>) -> bool
where
    T: EventListener,
{
    let in_docker_ssh = term.shell_state().in_docker | term.shell_state().in_ssh;
    let shell_enabled = [Some("bash"), Some("zsh"), Some("fish")]
        .contains(&term.shell_state().get_context().shell.as_deref());
    let prexec = term.shell_state().preexec;

    let mut handle = INSERTION_LOCKED_AT.lock();
    let insertion_locked = match handle.as_ref() {
        Some(at) => {
            let lock_expired =
                at.elapsed().unwrap_or_else(|_| Duration::new(0, 0)) > Duration::new(0, 50_000_000);
            let should_unlock = lock_expired
                || !term
                    .get_current_buffer()
                    .map(|buff| &buff.buffer == (&EXPECTED_BUFFER.lock() as &String))
                    .unwrap_or(true);
            if should_unlock {
                handle.take();
                if lock_expired {
                    trace!("insertion lock released because lock expired");
                    true
                } else {
                    trace!("insertion lock released because buffer looks like how we expect");
                    false
                }
            } else {
                false
            }
        }
        None => false,
    };
    drop(handle);

    trace!(
        "in_docker_ssh: {}, shell_enabled: {}, prexec: {}, insertion_locked: {}",
        in_docker_ssh,
        shell_enabled,
        prexec,
        insertion_locked
    );

    shell_enabled && !insertion_locked && !prexec
}

async fn send_edit_buffer<T>(term: &Term<T>, sender: &Sender<LocalMessage>) -> Result<()>
where
    T: EventListener,
{
    match term.get_current_buffer() {
        Some(edit_buffer) => {
            if let Some(cursor_idx) = edit_buffer.cursor_idx.and_then(|i| i.try_into().ok()) {
                info!("edit_buffer: {:?}", edit_buffer);

                let context = shell_state_to_context(term.shell_state());
                let edit_buffer_hook =
                    new_edit_buffer_hook(Some(context), edit_buffer.buffer, cursor_idx, 0);
                let message = hook_to_message(edit_buffer_hook);

                debug!("Sending: {:?}", message);

                sender.send_async(message).await?;
            }
            Ok(())
        }
        None => Err(anyhow!("No edit buffer to send")),
    }
}

async fn process_figterm_message(
    figterm_message: FigtermMessage,
    term: &Term<EventSender>,
    pty_master: &mut AsyncPtyMaster,
    mut intercept_set: DashSet<char, fnv::FnvBuildHasher>,
) -> Result<()> {
    match figterm_message.command {
        Some(figterm_message::Command::InsertTextCommand(command)) => {
            if let Some(ref text_to_insert) = command.insertion {
                if let Some((mut buffer, Some(mut position))) = term
                    .get_current_buffer()
                    .map(|buff| (buff.buffer, buff.cursor_idx))
                {
                    // perform deletion
                    if let Some(deletion) = command.deletion {
                        let deletion = deletion as usize;
                        buffer.drain(position - deletion + 1..=position);
                    }
                    // move cursor
                    if let Some(offset) = command.offset {
                        position += offset as usize;
                    }
                    // split text by cursor
                    let (left, right) = buffer.split_at(position);

                    INSERTION_LOCKED_AT.lock().replace(SystemTime::now());
                    let expected = format!("{}{}{}", left, text_to_insert, right);
                    trace!("lock set, expected buffer: {}", expected);
                    *EXPECTED_BUFFER.lock() = expected;
                }
            }
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
        args.push(CString::new("--login").expect("Failed to convert arg to CString"));
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

    execvp(&*args[0], &args).expect("Failed to execvp");
    unreachable!()
}

fn figterm_main() -> Result<()> {
    let term_session_id = env::var("TERM_SESSION_ID")
        .context("Failed to get TERM_SESSION_ID environment variable")?;

    logger::stdio_debug_log("Checking stdin fd is a tty");

    // Check that stdin is a tty
    if !isatty(STDIN_FILENO).context("Failed to check if stdin is a tty")? {
        anyhow::bail!("stdin is not a tty");
    }

    // Get term data
    let termios = tcgetattr(STDIN_FILENO).context("Failed to get terminal attributes")?;
    let old_termios = termios.clone();

    let mut winsize = get_winsize(STDIN_FILENO).context("Failed to get terminal size")?;

    logger::stdio_debug_log("Forking child shell process");

    // Fork pseudoterminal
    // SAFETY: forkpty is safe to call, but the child must not call any functions
    // that are not async-signal-safe.
    match fork_pty(&old_termios, &winsize).context("Failed to fork pty")? {
        PtyForkResult::Parent(pt_details, pid) => {
            let runtime = runtime::Builder::new_multi_thread()
                .enable_all()
                .thread_name("figterm-runtime-worker")
                .build()?;

            init_logger(&pt_details.pty_name).context("Failed to init logger")?;

            match runtime
                .block_on(async {
                    info!("Shell: {}", pid);
                    info!("Figterm: {}", getpid());
                    info!("Pty name: {}", pt_details.pty_name);

                    let history_sender = history::spawn_history_task().await;

                    let raw_termios = termios_to_raw(termios);
                    tcsetattr(STDIN_FILENO, SetArg::TCSAFLUSH, &raw_termios)?;
                    trace!("Set raw termios");

                    // Spawn thread to handle outgoing data to main Fig app
                    let outgoing_sender = spawn_outgoing_sender().await?;

                    // Spawn thread to handle incoming data
                    let incomming_receiver = spawn_incoming_receiver(&term_session_id).await?;

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
                    let mut write_buffer = [0u8; BUFFER_SIZE];

                    let intercept_set: DashSet<char, fnv::FnvBuildHasher> = DashSet::with_hasher(fnv::FnvBuildHasher::default());

                    // TODO: Write initial text to pty

                    let mut first_time = true;

                    'select_loop: loop {
                        if first_time && term.shell_state().has_seen_prompt {
                            trace!("Has seen prompt and first time");
                            let initial_command = env::var("FIG_START_TEXT").ok().filter(|s| !s.is_empty());
                            if let Some(mut initial_command) = initial_command {
                                debug!("Sending initial text: {}", initial_command);
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
                                match res {
                                    Ok(size) => match std::str::from_utf8(&read_buffer[..size]) {
                                            Ok(s) => {
                                                trace!("Read {} bytes from input: {:?}", size, s);
                                                let mut out = heapless::String::<BUFFER_SIZE>::new();
                                                for c in s.chars() {
                                                    if !intercept_set.contains(&c) {
                                                        // This should always be okay since the input <= BUFFER_SIZE
                                                        out.push(c).ok();
                                                    }
                                                }
                                                master.write(out.as_bytes()).await?;
                                            }
                                            Err(err) => {
                                                error!("Failed to convert utf8: {}", err);
                                                trace!("Read {} bytes from input: {:?}", size, &read_buffer[..size]);
                                                master.write(&read_buffer[..size]).await?;
                                            }
                                    },
                                    Err(err) => {
                                        error!("Failed to read from stdin: {}", err);
                                    }
                                }
                                Ok("stdin")
                            }
                            _ = window_change_signal.recv() => {
                                unsafe { read_winsize(STDIN_FILENO, &mut winsize) }?;
                                unsafe { ioctl_tiocswinsz(master.as_raw_fd(), &winsize) }?;
                                let window_size = SizeInfo::new(winsize.ws_row as usize, winsize.ws_col as usize);
                                debug!("Window size changed: {:?}", window_size);
                                term.resize(window_size);
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
                                    Err(err) => error!("Failed to read from master: {}", err),
                                }
                                Ok("master")
                            }
                            msg = incomming_receiver.recv_async() => {
                                match msg {
                                    Ok(buf) => {
                                        debug!("Received message from socket: {:?}", buf);
                                        process_figterm_message(buf, &term, &mut master, intercept_set.clone()).await?;
                                    }
                                    Err(err) => {
                                        error!("Failed to receive message from socket: {}", err);
                                    }
                                }
                                Ok("incomming_receiver")
                            }
                        };

                        if let Err(e) = select_result {
                            error!("Error in select loop: {}", e);
                            break 'select_loop;
                        }
                    }

                    remove_socket(&term_session_id).await?;

                    anyhow::Ok(())
                }) {
                    Ok(()) => {
                        if let Err(e) = tcsetattr(STDIN_FILENO, SetArg::TCSAFLUSH, &old_termios) {
                            error!("Failed to restore terminal settings: {}", e);
                        }

                        info!("Exiting");
                        exit(0);
                    },
                    Err(e) => {
                        if let Err(e) = tcsetattr(STDIN_FILENO, SetArg::TCSAFLUSH, &old_termios) {
                            error!("Failed to restore terminal settings: {}", e);
                        }

                        error!("Error in async runtime: {}", e);
                        Err(e)
                    },
                }
        }
        PtyForkResult::Child => {
            // DO NOT RUN ANY FUNCTIONS THAT ARE NOT ASYNC SIGNAL SAFE
            // https://man7.org/linux/man-pages/man7/signal-safety.7.html
            match launch_shell() {
                Ok(()) => Ok(()),
                Err(e) => {
                    println!("ERROR: {:?}", e);
                    capture_anyhow(&e);
                    Err(e)
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let _guard = sentry::init((
        "https://633267fac776481296eadbcc7093af4a@o436453.ingest.sentry.io/6187825",
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));

    sentry::configure_scope(|scope| {
        scope.set_user(Some(sentry::User {
            email: get_email(),
            ..Default::default()
        }));
    });

    Cli::parse();

    logger::stdio_debug_log(format!("FIG_LOG_LEVEL={}", logger::get_log_level()));

    if let Err(e) = figterm_main() {
        println!("Fig had an Error!: {:?}", e);
        capture_anyhow(&e);

        // Fallback to normal shell
        if let Err(e) = launch_shell() {
            capture_anyhow(&e);
            logger::stdio_debug_log(format!("{}", e));
        }
    }

    Ok(())
}
