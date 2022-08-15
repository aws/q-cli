pub mod cli;
mod event_handler;
pub mod history;
pub mod input;
pub mod interceptor;
pub mod ipc;
pub mod logger;
pub mod pty;
pub mod term;

use std::env;
#[cfg(unix)]
use std::ffi::{
    CString,
    OsStr,
};
use std::iter::repeat;
use std::sync::Arc;
use std::time::{
    Duration,
    SystemTime,
};

use alacritty_terminal::ansi::Processor;
use alacritty_terminal::event::EventListener;
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::term::{
    ShellState,
    SizeInfo,
    TextBuffer,
};
use alacritty_terminal::Term;
use anyhow::{
    anyhow,
    Context,
    Result,
};
use bytes::BytesMut;
use cfg_if::cfg_if;
use clap::StructOpt;
use cli::Cli;
use event_handler::EventHandler;
use fig_proto::hooks::{
    hook_to_message,
    new_edit_buffer_hook,
};
use fig_proto::local::{
    self,
    LocalMessage,
    TerminalCursorCoordinates,
};
use fig_settings::state;
use fig_telemetry::sentry::{
    capture_anyhow,
    release_name,
};
use fig_util::process_info::{
    Pid,
    PidExt,
};
use fig_util::Terminal as FigTerminal;
use flume::Sender;
#[cfg(unix)]
use nix::unistd::execvp;
use once_cell::sync::Lazy;
use parking_lot::{
    Mutex,
    RwLock,
};
use portable_pty::PtySize;
use tokio::io::{
    self,
    AsyncWriteExt,
};
use tokio::sync::oneshot;
use tokio::task::block_in_place;
use tokio::{
    runtime,
    select,
};
use tracing::{
    debug,
    error,
    info,
    trace,
    warn,
};

use crate::input::{
    InputEvent,
    KeyCode,
    KeyCodeEncodeModes,
    KeyboardEncoding,
};
use crate::interceptor::KeyInterceptor;
use crate::ipc::{
    remove_socket,
    spawn_incoming_receiver,
    spawn_outgoing_sender,
};
use crate::logger::init_logger;
#[cfg(unix)]
use crate::pty::unix::open_pty;
#[cfg(windows)]
use crate::pty::win::open_pty;
use crate::pty::CommandBuilder;
use crate::term::{
    SystemTerminal,
    Terminal,
};

const IS_FIG_PRO_KEY: &str = "user.account.is-fig-pro";

const BUFFER_SIZE: usize = 4096;

static INSERT_ON_NEW_CMD: Mutex<Option<String>> = Mutex::new(None);
static EXECUTE_ON_NEW_CMD: Mutex<bool> = Mutex::new(false);
static INSERTION_LOCKED_AT: RwLock<Option<SystemTime>> = RwLock::new(None);
static EXPECTED_BUFFER: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new("".to_string()));

fn shell_state_to_context(shell_state: &ShellState) -> local::ShellContext {
    let terminal = FigTerminal::parent_terminal().map(|s| s.to_string());

    let integration_version = std::env::var("FIG_INTEGRATION_VERSION")
        .map(|s| s.parse().ok())
        .ok()
        .flatten()
        .unwrap_or(8);

    let remote_context_type = if shell_state.in_ssh {
        Some(local::shell_context::RemoteContextType::Ssh)
    } else if shell_state.in_docker {
        Some(local::shell_context::RemoteContextType::Docker)
    } else {
        None
    };

    let remote_context = remote_context_type.is_some().then(|| {
        Box::new(local::ShellContext {
            pid: shell_state.remote_context.pid,
            ttys: shell_state.remote_context.tty.clone(),
            process_name: shell_state.remote_context.shell.clone(),
            shell_path: shell_state
                .remote_context
                .shell_path
                .clone()
                .map(|path| path.display().to_string()),
            wsl_distro: shell_state.remote_context.wsl_distro.clone(),
            current_working_directory: shell_state
                .remote_context
                .current_working_directory
                .clone()
                .map(|cwd| cwd.display().to_string()),
            session_id: shell_state.remote_context.session_id.clone(),
            integration_version: Some(integration_version),
            terminal: terminal.clone(),
            hostname: shell_state.remote_context.hostname.clone(),
            remote_context: None,
            remote_context_type: None,
        })
    });

    local::ShellContext {
        pid: shell_state.local_context.pid,
        ttys: shell_state.local_context.tty.clone(),
        process_name: shell_state.local_context.shell.clone(),
        shell_path: shell_state
            .local_context
            .shell_path
            .clone()
            .map(|path| path.display().to_string()),
        wsl_distro: shell_state.local_context.wsl_distro.clone(),
        current_working_directory: shell_state
            .local_context
            .current_working_directory
            .clone()
            .map(|cwd| cwd.display().to_string()),
        session_id: shell_state.local_context.session_id.clone(),
        integration_version: Some(integration_version),
        terminal,
        hostname: shell_state.local_context.hostname.clone(),
        remote_context,
        remote_context_type: remote_context_type.map(|x| x.into()),
    }
}

#[allow(clippy::needless_return)]
fn get_cursor_coordinates(terminal: &mut dyn Terminal) -> Option<TerminalCursorCoordinates> {
    cfg_if! {
        if #[cfg(target_os = "windows")] {
            use term::cast;

            let coordinate = terminal.get_cursor_coordinate().ok()?;
            let screen_size = terminal.get_screen_size().ok()?;
            return Some(TerminalCursorCoordinates {
                x: cast(coordinate.cols).ok()?,
                y: cast(coordinate.rows).ok()?,
                xpixel: cast(screen_size.xpixel).ok()?,
                ypixel: cast(screen_size.ypixel).ok()?,
            });
        } else {
            let _terminal = terminal;
            return None;
        }
    }
}

fn can_send_edit_buffer<T>(term: &Term<T>) -> bool
where
    T: EventListener,
{
    let in_docker_ssh = term.shell_state().in_docker | term.shell_state().in_ssh;
    let shell_enabled = [Some("bash"), Some("zsh"), Some("fish"), Some("nu")]
        .contains(&term.shell_state().get_context().shell.as_deref());
    let prexec = term.shell_state().preexec;

    let mut handle = INSERTION_LOCKED_AT.write();
    let insertion_locked = match handle.as_ref() {
        Some(at) => {
            let lock_expired = at.elapsed().unwrap_or_else(|_| Duration::new(0, 0)) > Duration::new(0, 50_000_000);
            let should_unlock = lock_expired
                || term
                    .get_current_buffer()
                    .map_or(true, |buff| &buff.buffer == (&EXPECTED_BUFFER.lock() as &String));
            if should_unlock {
                handle.take();
                if lock_expired {
                    trace!("insertion lock released because lock expired");
                } else {
                    trace!("insertion lock released because buffer looks like how we expect");
                }
                false
            } else {
                true
            }
        },
        None => false,
    };
    drop(handle);

    trace!(
        "in_docker_ssh: {in_docker_ssh}, shell_enabled: {shell_enabled}, prexec: {prexec}, insertion_locked: {insertion_locked}"
    );

    shell_enabled && !insertion_locked && !prexec
}

async fn send_edit_buffer<T>(
    term: &Term<T>,
    sender: &Sender<LocalMessage>,
    cursor_coordinates: Option<TerminalCursorCoordinates>,
) -> Result<()>
where
    T: EventListener,
{
    match term.get_current_buffer() {
        Some(edit_buffer) => {
            if let Some(cursor_idx) = edit_buffer.cursor_idx.and_then(|i| i.try_into().ok()) {
                debug!("edit_buffer: {edit_buffer:?}");
                trace!("buffer bytes: {:02X?}", edit_buffer.buffer.as_bytes());
                trace!("buffer chars: {:?}", edit_buffer.buffer.chars().collect::<Vec<_>>());

                let context = shell_state_to_context(term.shell_state());

                let edit_buffer_hook =
                    new_edit_buffer_hook(Some(context), edit_buffer.buffer, cursor_idx, 0, cursor_coordinates);
                let message = hook_to_message(edit_buffer_hook);

                debug!("Sending: {message:?}");

                sender.send_async(message).await?;
            }
            Ok(())
        },
        None => Err(anyhow!("No edit buffer to send")),
    }
}

fn get_parent_shell() -> Result<String> {
    match env::var("FIG_SHELL").ok().filter(|s| !s.is_empty()) {
        Some(v) => Ok(v),
        None => match env::var("SHELL").ok().filter(|s| !s.is_empty()) {
            Some(shell) => Ok(shell),
            None => {
                anyhow::bail!("No FIG_SHELL or SHELL found");
            },
        },
    }
}

fn build_shell_command() -> Result<CommandBuilder> {
    let parent_shell = get_parent_shell()?;
    let mut builder = CommandBuilder::new(&parent_shell);

    if env::var("FIG_IS_LOGIN_SHELL").ok().as_deref() == Some("1") {
        builder.arg("--login");
    }

    if let Some(execution_string) = env::var("FIG_EXECUTION_STRING").ok().filter(|s| !s.is_empty()) {
        builder.args(["-c", &execution_string]);
    }

    if let Some(extra_args) = env::var("FIG_SHELL_EXTRA_ARGS").ok().filter(|s| !s.is_empty()) {
        builder.args(extra_args.split_whitespace().filter(|arg| arg != &"--login"));
    }

    builder.env("FIG_TERM", "1");
    builder.env("FIG_TERM_VERSION", env!("CARGO_PKG_VERSION"));
    if env::var_os("TMUX").is_some() {
        builder.env("FIG_TERM_TMUX", "1");
    }

    // Clean up environment and launch shell.
    builder.env_remove("FIG_SHELL");
    builder.env_remove("FIG_IS_LOGIN_SHELL");
    builder.env_remove("FIG_START_TEXT");
    builder.env_remove("FIG_SHELL_EXTRA_ARGS");
    builder.env_remove("FIG_EXECUTION_STRING");

    if let Ok(dir) = std::env::current_dir() {
        builder.cwd(dir);
    }

    Ok(builder)
}

#[cfg(unix)]
fn launch_shell() -> Result<()> {
    let cmd = build_shell_command()?.as_command()?;
    let mut args: Vec<&OsStr> = std::vec![cmd.get_program()];
    args.extend(cmd.get_args());

    let cargs: Vec<_> = args
        .into_iter()
        .map(|arg| CString::new(arg.to_string_lossy().as_ref()).expect("Failed to convert arg to CString"))
        .collect();
    for (key, val) in cmd.get_envs() {
        match val {
            Some(value) => env::set_var(key, value),
            None => {
                env::remove_var(key);
            },
        }
    }

    execvp(&cargs[0], &cargs).expect("Failed to execvp");
    unreachable!()
}

fn figterm_main() -> Result<()> {
    let term_session_id = env::var("TERM_SESSION_ID").context("Failed to get TERM_SESSION_ID environment variable")?;
    let mut terminal = SystemTerminal::new_from_stdio()?;
    let screen_size = terminal.get_screen_size()?;

    let pty_size = PtySize {
        rows: screen_size.rows as u16,
        cols: screen_size.cols as u16,
        pixel_width: screen_size.xpixel as u16,
        pixel_height: screen_size.ypixel as u16,
    };

    let pty = open_pty(&pty_size).context("Failed to open pty")?;
    let command = build_shell_command()?;

    let pty_name = pty.slave.get_name().unwrap_or_else(|| term_session_id.clone());
    logger::stdio_debug_log(format!("pty name: {}", pty_name));
    init_logger(&pty_name).context("Failed to init logger")?;
    logger::stdio_debug_log("Forking child shell process");
    let mut child = pty.slave.spawn_command(command)?;
    let (child_tx, mut child_rx) = oneshot::channel();
    info!("Shell: {:?}", child.process_id());
    std::thread::spawn(move || child_tx.send(child.wait()));
    info!("Figterm: {}", Pid::current());
    info!("Pty name: {pty_name}");

    terminal.set_raw_mode()?;

    let runtime = runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name_fn(|| {
            static ATOMIC_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let id = ATOMIC_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            format!("figterm-runtime-worker-{id}")
        })
        .build()?;
    let runtime_result = runtime.block_on(async {
        let history_sender = history::spawn_history_task().await;
        // Spawn thread to handle outgoing data to main Fig app
        let outgoing_sender = spawn_outgoing_sender().await?;

        // Spawn thread to handle incoming data
        let incoming_receiver = spawn_incoming_receiver(&term_session_id).await?;

        let mut stdout = io::stdout();
        let mut master = pty.master.get_async_master_pty()?;

        let mut processor = Processor::new();
        let size = SizeInfo::new(pty_size.rows as usize, pty_size.cols as usize);

        let (event_sender_tx, event_sender_rx) = flume::bounded(16);
        let event_sender = EventHandler::new(outgoing_sender.clone(), history_sender, event_sender_tx);

        let mut term = alacritty_terminal::Term::new(size, event_sender, 1);

        #[cfg(windows)]
        term.set_windows_delay_end_prompt(true);

        let mut write_buffer: Vec<u8> = vec![0; BUFFER_SIZE];

        let mut key_interceptor = KeyInterceptor::new();
        key_interceptor.load_key_intercepts()?;

        let mut first_time = true;

        let mut edit_buffer_interval = tokio::time::interval(Duration::from_millis(16));

        let input_rx = terminal.read_input()?;

        let modes = KeyCodeEncodeModes {
            #[cfg(unix)]
            encoding: KeyboardEncoding::Xterm,
            #[cfg(windows)]
            encoding: KeyboardEncoding::Win32,
            application_cursor_keys: false,
            newline_mode: false,
        };

        let fig_pro = Arc::new(Mutex::new(false));
        let fig_pro_clone = fig_pro.clone();
        tokio::spawn(async move {
            *fig_pro_clone.lock() = block_in_place(|| fig_settings::state::get_bool_or(IS_FIG_PRO_KEY, false));
            let fig_pro = fig_api_client::user::plans()
                .await
                .map(|plans| plans.highest_plan())
                .unwrap_or_default()
                .is_pro();
            *fig_pro_clone.lock() = fig_pro;
            block_in_place(|| fig_settings::state::set_value(IS_FIG_PRO_KEY, fig_pro).ok());
        });

        let ai_enabled = fig_settings::settings::get_bool_or("ai.terminal-hash-sub", true);

        let result: Result<()> = 'select_loop: loop {
            if first_time && term.shell_state().has_seen_prompt {
                trace!("Has seen prompt and first time");
                let initial_command = env::var("FIG_START_TEXT").ok().filter(|s| !s.is_empty());
                if let Some(mut initial_command) = initial_command {
                    debug!("Sending initial text: {initial_command}");
                    initial_command.push('\n');
                    if let Err(err) = master.write(initial_command.as_bytes()).await {
                        error!("Failed to write initial command: {err}");
                    }
                }
                first_time = false;
            }

            let select_result: Result<()> = select! {
                biased;
                res = input_rx.recv_async() => {
                    let mut input_res = Ok(());
                    match res {
                        Ok(events) => {
                            let mut write_buffer = BytesMut::new();
                            for event in events {
                                match event {
                                    Ok((raw, InputEvent::Key(event))) => {
                                        info!("Got key, {event:?}, {raw:?}");
                                        if ai_enabled && *fig_pro.lock() && event.key == KeyCode::Enter && event.modifiers == input::Modifiers::NONE {
                                            if let Some(TextBuffer { buffer, cursor_idx }) = term.get_current_buffer() {
                                                let buffer = buffer.trim();
                                                if buffer.len() > 1 && buffer.starts_with('#') && term.columns() > buffer.len() {
                                                    write_buffer.extend(
                                                        &repeat(b'\x08')
                                                            .take(buffer.len()
                                                            .max(cursor_idx.unwrap_or(0)))
                                                            .collect::<Vec<_>>()
                                                    );
                                                    write_buffer.extend(
                                                        format!(
                                                            "fig ai '{}'\r",
                                                            buffer
                                                                .trim_start_matches('#')
                                                                .trim()
                                                                .replace('\'', "'\"'\"'")
                                                            ).as_bytes()
                                                    );
                                                    master.write(&write_buffer).await?;
                                                    continue 'select_loop;
                                                }
                                            }
                                        }

                                        let raw = raw.or_else(|| {
                                            event.key.encode(event.modifiers, modes, true)
                                                .ok()
                                                .map(|s| s.into_bytes().into())
                                        });

                                        if let Some(action) = key_interceptor.intercept_key(&event) {
                                            debug!("Intercepted action: {action:?}");
                                            let s = raw.clone()
                                                .and_then(|b| String::from_utf8(b.to_vec()).ok())
                                                .unwrap_or_default();
                                            let hook = fig_proto::hooks::new_intercepted_key_hook(None, action.to_string(), s);
                                            outgoing_sender.send(hook_to_message(hook)).unwrap();

                                            if event.key == KeyCode::Escape {
                                                key_interceptor.reset();
                                            }
                                        } else if let Some(bytes) = raw {
                                            write_buffer.extend(&bytes);
                                        }
                                    }
                                    Ok((_, InputEvent::Resized)) => {
                                        terminal.flush()?;

                                        let size = terminal.get_screen_size()?;
                                        let pty_size = PtySize {
                                            rows: size.rows as u16,
                                            cols: size.cols as u16,
                                            pixel_width: size.xpixel as u16,
                                            pixel_height: size.ypixel as u16,
                                        };

                                        master.resize(pty_size)?;
                                        let window_size = SizeInfo::new(size.rows as usize, size.cols as usize);
                                        debug!("Window size changed: {window_size:?}");
                                        term.resize(window_size);
                                    }
                                    Ok((None, InputEvent::Paste(string))) => {
                                        info!("Pasty");
                                        // Pass through bracketed pastes.
                                        write_buffer.extend(b"\x1b[200~");
                                        write_buffer.extend(string.as_bytes());
                                        write_buffer.extend(b"\x1b[201~");
                                    }
                                    Ok((raw, _)) => {
                                        if let Some(raw) = raw {
                                            info!("Fallback write");
                                            write_buffer.extend(&raw);
                                        } else {
                                            info!("Unhandled input event with no raw pass-through data");
                                        }
                                    }
                                    Err(err) => {
                                        error!("Failed receiving input from stdin: {err}");
                                        input_res = Err(err);
                                        break;
                                    }
                                };
                            }
                            master.write(&write_buffer).await?;
                        }
                        Err(err) => {
                            warn!("Failed recv: {err}");
                        }
                    };
                    input_res
                }
                res = event_sender_rx.recv_async() => {
                    match res {
                        Ok(b) => {
                            master.write(&b).await?;
                        }
                        Err(err) => warn!("Failed to recv: {err}"),
                    };
                    Ok(())
                }
                res = master.read(&mut write_buffer) => {
                    match res {
                        Ok(0) => {
                            trace!("EOF from master");
                            break 'select_loop Ok(());
                        }
                        Ok(size) => {
                            trace!("Read {size} bytes from master");

                            let old_delayed_count = term.get_delayed_events_count();
                            for byte in &write_buffer[..size] {
                                processor.advance(&mut term, *byte);
                            }

                            let delayed_count = term.get_delayed_events_count();

                            // We have delayed events and did not receive delayed events. Flush all
                            // delayed events now.
                            if delayed_count > 0 && delayed_count == old_delayed_count {
                                term.flush_delayed_events();
                            }

                            stdout.write_all(&write_buffer[..size]).await?;
                            stdout.flush().await?;

                            if write_buffer.capacity() == write_buffer.len() {
                                write_buffer.reserve(write_buffer.len());
                            }

                            if can_send_edit_buffer(&term) {
                                let cursor_coordinates = get_cursor_coordinates(&mut terminal);
                                if let Err(err) = send_edit_buffer(&term, &outgoing_sender, cursor_coordinates).await {
                                    warn!("Failed to send edit buffer: {err}");
                                }
                            }

                            Ok(())
                        }
                        Err(err) => {
                            error!("Failed to read from master: {err}");
                            break 'select_loop Ok(());
                        }
                    }
                }
                msg = incoming_receiver.recv_async() => {
                    match msg {
                        Ok((message, sender)) => {
                            debug!("Received message from socket: {message:?}");
                            ipc::process_figterm_message(message, sender, &term, &mut master, &mut key_interceptor).await?;
                        }
                        Err(err) => {
                            error!("Failed to receive message from socket: {err}");
                        }
                    }
                    Ok(())
                }
                // Check if to send the edit buffer because of timeout
                _ = edit_buffer_interval.tick() => {
                    let send_eb = INSERTION_LOCKED_AT.read().is_some();
                    if send_eb && can_send_edit_buffer(&term) {
                        let cursor_coordinates = get_cursor_coordinates(&mut terminal);
                        if let Err(err) = send_edit_buffer(&term, &outgoing_sender, cursor_coordinates).await {
                            warn!("Failed to send edit buffer: {err}");
                        }
                    }
                    Ok(())
                }
                _ = &mut child_rx => {
                    trace!("Shell process exited");
                    break 'select_loop Ok(());
                }
            };

            if let Err(err) = select_result {
                error!("Error in select loop: {err}");
                break 'select_loop Err(err);
            }
        };

        remove_socket(&term_session_id).await?;

        result
    });

    // Reading from stdin is a blocking task on a separate thread:
    // https://github.com/tokio-rs/tokio/issues/2466
    // We must explicitly shutdown the runtime to exit.
    // This can cause resource leaks if we aren't careful about tasks we spawn.
    runtime.shutdown_background();

    runtime_result
}

fn main() {
    let _guard = fig_telemetry::init_sentry(
        release_name!(),
        "https://633267fac776481296eadbcc7093af4a@o436453.ingest.sentry.io/6187825",
        1.0,
        false,
    );

    Cli::parse();

    logger::stdio_debug_log(format!("FIG_LOG_LEVEL={}", logger::get_log_level()));

    if !state::get_bool_or("figterm.enabled", true) {
        println!("[NOTE] figterm is disabled. Autocomplete will not work.");
        logger::stdio_debug_log("figterm is disabled. `figterm.enabled` == false");
        return;
    }

    match figterm_main() {
        Ok(()) => {
            info!("Exiting");
        },
        Err(err) => {
            error!("Error in async runtime: {err}");
            println!("Fig had an Error!: {err:?}");
            capture_anyhow(&err);

            // Fallback to normal shell
            #[cfg(unix)]
            if let Err(err) = launch_shell() {
                capture_anyhow(&err);
                logger::stdio_debug_log(err.to_string());
            }
        },
    }
}
