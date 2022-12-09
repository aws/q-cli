use std::ffi::OsStr;
use std::iter::repeat;
use std::path::{
    Path,
    PathBuf,
};
use std::process::Command;
use std::time::{
    Duration,
    SystemTime,
};

use alacritty_terminal::term::ShellState;
use alacritty_terminal::Term;
use anyhow::Result;
use fig_api_client::ai::EditBufferComponent;
use fig_proto::fig::{
    EnvironmentVariable,
    PseudoterminalExecuteResponse,
    RunProcessResponse,
};
use fig_proto::figterm::figterm_request_message::Request as FigtermRequest;
use fig_proto::figterm::figterm_response_message::Response as FigtermResponse;
use fig_proto::figterm::intercept_request::{
    InterceptCommand,
    SetFigjsIntercepts,
    SetFigjsVisible,
};
use fig_proto::figterm::{
    self,
    CodexCompleteResponse,
    FigtermRequestMessage,
    FigtermResponseMessage,
};
use fig_proto::secure::{
    clientbound,
    hostbound,
    Clientbound,
    Hostbound,
};
use fig_util::directories::home_dir_utf8;
use flume::Sender;
use once_cell::sync::Lazy;
use tracing::{
    debug,
    error,
    info,
    trace,
    warn,
};

use crate::event_handler::EventHandler;
use crate::history::{
    HistoryQueryParams,
    HistorySender,
};
use crate::interceptor::KeyInterceptor;
use crate::pty::AsyncMasterPty;
use crate::{
    history,
    shell_state_to_context,
    MainLoopEvent,
    EXECUTE_ON_NEW_CMD,
    EXPECTED_BUFFER,
    INSERTION_LOCKED_AT,
    INSERT_ON_NEW_CMD,
    SHELL_ENVIRONMENT_VARIABLES,
};

fn shell_args(shell_path: &str) -> &'static [&'static str] {
    let (_, shell_name) = shell_path
        .rsplit_once(|c| c == '/' || c == '\\')
        .unwrap_or(("", shell_path));
    match shell_name {
        "bash" | "bash.exe" => &["--norc", "--noprofile", "-c"],
        "zsh" => &["--norcs", "-c"],
        "fish" => &["--no-config", "-c"],
        _ => {
            warn!("unknown shell {shell_name}");
            &[]
        },
    }
}

fn working_directory(path: Option<&str>, shell_state: &ShellState) -> PathBuf {
    let map_dir = |path: PathBuf| match path.canonicalize() {
        Ok(path) if path.is_dir() => Some(path),
        Ok(path) => {
            warn!(?path, "not a directory");
            None
        },
        Err(err) => {
            warn!(?path, %err, "failed to canonicalize path");
            None
        },
    };

    path.map(PathBuf::from)
        .and_then(map_dir)
        .or_else(|| {
            shell_state
                .get_context()
                .current_working_directory
                .clone()
                .and_then(map_dir)
        })
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| {
            cfg_if::cfg_if! {
                if #[cfg(windows)] {
                    PathBuf::from("C:\\")
                } else if #[cfg(unix)] {
                    PathBuf::from("/")
                }
            }
        })
}

fn create_command(executable: impl AsRef<Path>, working_directory: impl AsRef<Path>) -> Command {
    let env = (*SHELL_ENVIRONMENT_VARIABLES.lock())
        .clone()
        .into_iter()
        .filter_map(|EnvironmentVariable { key, value }| value.map(|value| (key, value)))
        .collect::<Vec<_>>();

    let mut cmd = if executable.as_ref().is_absolute() {
        Command::new(executable.as_ref())
    } else {
        let path = env
            .iter()
            .find_map(|(key, value)| if key == "PATH" { Some(value.as_str()) } else { None });

        which::which_in(executable.as_ref(), path, working_directory.as_ref())
            .map(Command::new)
            .unwrap_or_else(|_| Command::new(executable.as_ref()))
    };

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(windows::Win32::System::Threading::DETACHED_PROCESS.0);
    }

    cmd.current_dir(working_directory);

    #[cfg(unix)]
    {
        use std::io;
        use std::os::unix::process::CommandExt;

        use nix::libc;

        // SAFETY: this closure is run after forking the process and only affects the
        // child. setsid is async-signal-safe.
        unsafe {
            cmd.pre_exec(|| {
                // Remove controlling terminal.
                if libc::setsid() == -1 {
                    Err(io::Error::last_os_error())
                } else {
                    Ok(())
                }
            });
        }
    }

    if !env.is_empty() {
        cmd.env_clear();
        cmd.envs(env);
    }

    cmd.env_remove("LS_COLORS");
    cmd.env_remove("CLICOLOR_FORCE");
    cmd.env_remove("CLICOLOR");
    cmd.env_remove("COLORTERM");
    cmd.envs([
        ("PROCESS_LAUNCHED_BY_FIG", "1"),
        ("HISTFILE", ""),
        ("HISTCONTROL", "ignoreboth"),
        ("TERM", "xterm-256color"),
        ("NO_COLOR", "1"),
    ]);

    cmd
}

/// Process the inner figterm request enum, shared between local and secure
pub async fn process_figterm_request(
    figterm_request: FigtermRequest,
    main_loop_tx: Sender<MainLoopEvent>,
    term: &Term<EventHandler>,
    pty_master: &mut Box<dyn AsyncMasterPty + Send + Sync>,
    key_interceptor: &mut KeyInterceptor,
) -> Result<Option<FigtermResponse>> {
    match figterm_request {
        FigtermRequest::InsertText(request) => {
            // If the shell is in prompt or a command is being executed, insert the text only
            // if the insert during command option is enabled.
            if term.shell_state().preexec && !request.insert_during_command() {
                return Ok(None);
            }

            let current_buffer = term.get_current_buffer().map(|buff| (buff.buffer, buff.cursor_idx));
            let mut insertion_string = String::new();
            if let Some((buffer, Some(position))) = current_buffer {
                if let Some(ref text_to_insert) = request.insertion {
                    trace!(?buffer, ?position);

                    // perform deletion
                    // if let Some(deletion) = command.deletion {
                    //     let deletion = deletion as usize;
                    //     buffer.drain(position - deletion..position);
                    // }
                    // // move cursor
                    // if let Some(offset) = command.offset {
                    //     position += offset as usize;
                    // }
                    // // split text by cursor
                    // let (left, right) = buffer.split_at(position);

                    INSERTION_LOCKED_AT.write().replace(SystemTime::now());
                    let expected = format!("{buffer}{text_to_insert}");
                    trace!(?expected, "lock set, expected buffer");
                    *EXPECTED_BUFFER.lock() = expected;
                }
                if let Some(ref insertion_buffer) = request.insertion_buffer {
                    if buffer.ne(insertion_buffer) {
                        if buffer.starts_with(insertion_buffer) {
                            if let Some(len_diff) = buffer.len().checked_sub(insertion_buffer.len()) {
                                insertion_string.extend(repeat('\x08').take(len_diff));
                            }
                        } else if insertion_buffer.starts_with(&buffer) {
                            insertion_string.push_str(&insertion_buffer[buffer.len()..]);
                        }
                    }
                }
            }
            insertion_string.push_str(&request.to_term_string());
            pty_master.write(insertion_string.as_bytes()).await?;
            Ok(None)
        },
        FigtermRequest::Intercept(request) => {
            match request.intercept_command {
                Some(InterceptCommand::SetFigjsIntercepts(SetFigjsIntercepts {
                    intercept_bound_keystrokes,
                    intercept_global_keystrokes,
                    actions,
                    override_actions,
                })) => {
                    key_interceptor.set_intercept_global(intercept_global_keystrokes);
                    key_interceptor.set_intercept(intercept_bound_keystrokes);
                    key_interceptor.set_actions(&actions, override_actions);
                },
                Some(InterceptCommand::SetFigjsVisible(SetFigjsVisible { visible })) => {
                    key_interceptor.set_window_visible(visible);
                },
                None => {},
            }

            Ok(None)
        },
        FigtermRequest::Diagnostics(_) => {
            let map_color = |color: &fig_color::VTermColor| -> figterm::TermColor {
                figterm::TermColor {
                    color: Some(match color {
                        fig_color::VTermColor::Rgb(r, g, b) => {
                            figterm::term_color::Color::Rgb(figterm::term_color::Rgb {
                                r: *r as i32,
                                b: *b as i32,
                                g: *g as i32,
                            })
                        },
                        fig_color::VTermColor::Indexed(i) => figterm::term_color::Color::Indexed(*i as u32),
                    }),
                }
            };

            let map_style = |style: &fig_color::SuggestionColor| -> figterm::TermStyle {
                figterm::TermStyle {
                    fg: style.fg().as_ref().map(map_color),
                    bg: style.bg().as_ref().map(map_color),
                }
            };

            let (edit_buffer, cursor_position) = term
                .get_current_buffer()
                .map(|buf| (Some(buf.buffer), buf.cursor_idx.and_then(|i| i.try_into().ok())))
                .unwrap_or((None, None));

            let response = FigtermResponse::Diagnostics(figterm::DiagnosticsResponse {
                shell_context: Some(shell_state_to_context(term.shell_state())),
                fish_suggestion_style: term.shell_state().fish_suggestion_color.as_ref().map(map_style),
                zsh_autosuggestion_style: term.shell_state().zsh_autosuggestion_color.as_ref().map(map_style),
                edit_buffer,
                cursor_position,
            });

            Ok(Some(response))
        },
        FigtermRequest::InsertOnNewCmd(command) => {
            *INSERT_ON_NEW_CMD.lock() = Some(command.text);
            *EXECUTE_ON_NEW_CMD.lock() = command.execute;
            Ok(None)
        },
        FigtermRequest::SetBuffer(_) => Err(anyhow::anyhow!("SetBuffer is not supported in figterm")),
        FigtermRequest::UpdateShellContext(request) => {
            if request.update_environment_variables {
                *SHELL_ENVIRONMENT_VARIABLES.lock() = request.environment_variables;
            }
            Ok(None)
        },
        FigtermRequest::NotifySshSessionStarted(notification) => {
            main_loop_tx.send(MainLoopEvent::PromptSSH(notification.uuid)).ok();
            Ok(None)
        },
        FigtermRequest::CodexComplete(_) => anyhow::bail!("CodexComplete is not supported over secure"),
    }
}

static LAST_RECEIVED: Lazy<tokio::sync::Mutex<Option<SystemTime>>> = Lazy::new(|| tokio::sync::Mutex::new(None));

static CACHE_ENABLED: Lazy<bool> = Lazy::new(|| std::env::var_os("FIG_CODEX_CACHE_DISABLE").is_none());
pub static COMPLETION_CACHE: Lazy<moka::sync::Cache<String, Option<String>>> = Lazy::new(|| {
    moka::sync::Cache::builder()
        .max_capacity(50)
        .time_to_idle(Duration::from_secs(30))
        .time_to_live(Duration::from_secs(60 * 5))
        .build()
});

/// Process a figterm request message
#[allow(clippy::too_many_arguments)]
pub async fn process_figterm_message(
    figterm_request_message: FigtermRequestMessage,
    main_loop_tx: Sender<MainLoopEvent>,
    response_tx: Sender<FigtermResponseMessage>,
    term: &Term<EventHandler>,
    history_sender: &HistorySender,
    pty_master: &mut Box<dyn AsyncMasterPty + Send + Sync>,
    key_interceptor: &mut KeyInterceptor,
    session_id: &str,
) -> Result<()> {
    match figterm_request_message.request {
        Some(FigtermRequest::CodexComplete(request)) => {
            let history_sender = history_sender.clone();
            let session_id = session_id.to_owned();
            tokio::spawn(async move {
                let figterm_request = request;

                if *CACHE_ENABLED {
                    // use cached completion if available
                    if let Some(insert_text) = COMPLETION_CACHE.get(&figterm_request.buffer) {
                        if let Err(err) = response_tx
                            .send_async(FigtermResponseMessage {
                                response: Some(FigtermResponse::CodexComplete(CodexCompleteResponse { insert_text })),
                            })
                            .await
                        {
                            error!(%err, "Failed to send codex completion");
                        }
                        return;
                    }
                }

                // debounce requests
                let debounce_duration = Duration::from_millis(
                    std::env::var("FIG_CODEX_DEBOUNCE_MS")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(300),
                );

                let now = SystemTime::now();
                LAST_RECEIVED.lock().await.replace(now);

                tokio::time::sleep(debounce_duration).await;
                if *LAST_RECEIVED.lock().await == Some(now) {
                    // TODO: determine behavior here, None or Some(unix timestamp)
                    *LAST_RECEIVED.lock().await = Some(SystemTime::now());
                } else {
                    warn!("Received another codex completion request, aborting");
                    if let Err(err) = response_tx
                        .send_async(FigtermResponseMessage {
                            response: Some(FigtermResponse::CodexComplete(CodexCompleteResponse {
                                insert_text: None,
                            })),
                        })
                        .await
                    {
                        error!(%err, "Failed to send codex completion");
                    }

                    return;
                }

                info!("Sending codex completion request");

                let (history_query_tx, history_query_rx) = flume::bounded(1);
                if let Err(err) = history_sender
                    .send_async(history::HistoryCommand::Query(
                        HistoryQueryParams {
                            limit: std::env::var("FIG_CODEX_HISTORY_COUNT")
                                .ok()
                                .and_then(|s| s.parse().ok())
                                .unwrap_or(25),
                        },
                        history_query_tx,
                    ))
                    .await
                {
                    error!(%err, "Failed to send history query");
                }

                let history = match history_query_rx.recv_async().await {
                    Ok(Some(history)) => history,
                    err => {
                        error!(?err, "Failed to get history");
                        vec![]
                    },
                };

                let request = fig_api_client::ai::CodexRequest {
                    history: history
                        .into_iter()
                        .map(|entry| fig_api_client::ai::CommandInfo {
                            command: entry.command,
                            cwd: entry.cwd,
                            time: entry.start_time.map(|t| t.into()),
                            exit_code: entry.exit_code,
                            hostname: entry.hostname,
                            pid: entry.pid,
                            session_id: entry.session_id,
                            shell: entry.shell,
                        })
                        .collect::<Vec<_>>(),
                    os: std::env::consts::OS.to_string(),
                    arch: std::env::consts::ARCH.to_string(),
                    time: Some(time::OffsetDateTime::now_utc()),
                    cwd: std::env::current_dir()
                        .ok()
                        .and_then(|p| p.to_str().map(|s| s.to_string())),
                    edit_buffer: vec![
                        EditBufferComponent::String(figterm_request.buffer.clone()),
                        EditBufferComponent::Other {
                            r#type: "cursor".to_string(),
                        },
                    ],
                    home_dir: home_dir_utf8().map(|s| s.into()).ok(),
                    session_id: Some(session_id),
                };

                let response = fig_api_client::ai::request(request).await;

                let insert_text = match response {
                    Ok(response) => {
                        if response.accuracy_rating.unwrap_or(1.0)
                            < std::env::var("FIG_CODEX_ACCURACY_THRESHOLD")
                                .ok()
                                .and_then(|s| s.parse().ok())
                                .unwrap_or(0.5)
                        {
                            None
                        } else {
                            response.insert_text
                        }
                    },
                    Err(err) => {
                        error!(%err, "Failed to get codex completion");
                        None
                    },
                };

                info!(?insert_text, "Got codex completion");

                if *CACHE_ENABLED {
                    COMPLETION_CACHE.insert(figterm_request.buffer, insert_text.clone());
                }

                if let Err(err) = response_tx
                    .send_async(FigtermResponseMessage {
                        response: Some(FigtermResponse::CodexComplete(CodexCompleteResponse { insert_text })),
                    })
                    .await
                {
                    error!(%err, "Failed to send codex completion");
                }
            });
        },
        Some(request) => {
            match process_figterm_request(request, main_loop_tx, term, pty_master, key_interceptor).await {
                Ok(Some(response)) => {
                    let response_message = FigtermResponseMessage {
                        response: Some(response),
                    };
                    if let Err(err) = response_tx.send_async(response_message).await {
                        error!(%err, "Failed sending request response");
                    }
                },
                Ok(None) => {},
                Err(err) => error!(%err, "Failed to process figterm message"),
            }
        },
        None => warn!("Figterm message with no request"),
    }
    Ok(())
}

async fn send_figterm_response_hostbound(
    response: Option<FigtermResponse>,
    nonce: Option<u64>,
    response_tx: &Sender<Hostbound>,
) {
    use hostbound::response::Response;

    if let Some(response) = response {
        let hostbound = Hostbound {
            packet: Some(hostbound::Packet::Response(hostbound::Response {
                nonce,
                response: Some(match response {
                    FigtermResponse::Diagnostics(diagnostics) => Response::Diagnostics(diagnostics),
                    FigtermResponse::CodexComplete(_codex_complete) => unimplemented!(),
                }),
            })),
        };

        if let Err(err) = response_tx.send_async(hostbound).await {
            error!(%err, "Failed sending request response");
        }
    }
}

pub async fn process_secure_message(
    clientbound_message: Clientbound,
    main_loop_tx: Sender<MainLoopEvent>,
    response_tx: Sender<Hostbound>,
    term: &Term<EventHandler>,
    pty_master: &mut Box<dyn AsyncMasterPty + Send + Sync>,
    key_interceptor: &mut KeyInterceptor,
) -> Result<()> {
    use clientbound::request::Request;
    use hostbound::response::Response;

    match clientbound_message.packet {
        Some(clientbound::Packet::Request(request)) => {
            let nonce = request.nonce;
            let make_response = move |response: Response| -> Hostbound {
                Hostbound {
                    packet: Some(hostbound::Packet::Response(hostbound::Response {
                        response: Some(response),
                        nonce,
                    })),
                }
            };

            match request.request {
                Some(Request::InsertText(request)) => {
                    send_figterm_response_hostbound(
                        process_figterm_request(
                            FigtermRequest::InsertText(request),
                            main_loop_tx,
                            term,
                            pty_master,
                            key_interceptor,
                        )
                        .await?,
                        nonce,
                        &response_tx,
                    )
                    .await;
                },
                Some(Request::Intercept(request)) => {
                    send_figterm_response_hostbound(
                        process_figterm_request(
                            FigtermRequest::Intercept(request),
                            main_loop_tx,
                            term,
                            pty_master,
                            key_interceptor,
                        )
                        .await?,
                        nonce,
                        &response_tx,
                    )
                    .await;
                },
                Some(Request::Diagnostics(request)) => {
                    send_figterm_response_hostbound(
                        process_figterm_request(
                            FigtermRequest::Diagnostics(request),
                            main_loop_tx,
                            term,
                            pty_master,
                            key_interceptor,
                        )
                        .await?,
                        nonce,
                        &response_tx,
                    )
                    .await;
                },
                Some(Request::InsertOnNewCmd(request)) => {
                    send_figterm_response_hostbound(
                        process_figterm_request(
                            FigtermRequest::InsertOnNewCmd(request),
                            main_loop_tx,
                            term,
                            pty_master,
                            key_interceptor,
                        )
                        .await?,
                        nonce,
                        &response_tx,
                    )
                    .await;
                },
                Some(Request::RunProcess(request)) => {
                    // TODO(sean) we can infer shell as above for execute if no executable is provided.
                    let mut cmd = create_command(
                        &request.executable,
                        working_directory(request.working_directory.as_deref(), term.shell_state()),
                    );

                    cmd.args(request.arguments);
                    for var in request.env {
                        cmd.env(var.key.clone(), var.value());
                    }

                    tokio::spawn(async move {
                        debug!("running command");
                        match cmd.output() {
                            Ok(output) => {
                                debug!("command successfully ran");
                                let response = make_response(Response::RunProcess(RunProcessResponse {
                                    stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                                    exit_code: output.status.code().unwrap_or(0),
                                }));
                                if let Err(err) = response_tx.send_async(response).await {
                                    error!(%err, "Failed sending request response");
                                }
                            },
                            Err(err) => {
                                debug!("command unsuccessfully ran");
                                warn!(%err, command = request.executable, "Failed running command");
                            },
                        }
                    });
                },
                Some(Request::PseudoterminalExecute(request)) => {
                    let default_command_shell = term
                        .shell_state()
                        .local_context
                        .shell_path
                        .as_ref()
                        .map(|x| x.as_os_str())
                        .unwrap_or_else(|| OsStr::new("/bin/bash"))
                        .to_owned();

                    let mut cmd = create_command(
                        &default_command_shell,
                        working_directory(request.working_directory.as_deref(), term.shell_state()),
                    );
                    // TODO(sean): better SHELL_ARGs handling here based on shell.
                    let args = shell_args(&default_command_shell.to_string_lossy());
                    cmd.args(args);
                    cmd.arg(&request.command);

                    for EnvironmentVariable { key, value } in &request.env {
                        match value {
                            Some(value) => cmd.env(key, value),
                            None => cmd.env_remove(key),
                        };
                    }

                    tokio::spawn(async move {
                        debug!("pseudoterminal executing");
                        match cmd.output() {
                            Err(err) => {
                                warn!(%err, command = request.command, "Failed running command");
                            },
                            Ok(output) => {
                                let response =
                                    make_response(Response::PseudoterminalExecute(PseudoterminalExecuteResponse {
                                        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                                        stderr: if output.stderr.is_empty() {
                                            None
                                        } else {
                                            Some(String::from_utf8_lossy(&output.stderr).to_string())
                                        },
                                        exit_code: output.status.code(),
                                    }));
                                if let Err(err) = response_tx.send_async(response).await {
                                    error!(%err, "Failed sending request response");
                                }
                            },
                        }
                    });
                },
                _ => warn!("unhandled request {request:?}"),
            }
        },
        Some(clientbound::Packet::Ping(())) => {
            let response = Hostbound {
                packet: Some(hostbound::Packet::Pong(())),
            };

            if let Err(err) = response_tx.send_async(response).await {
                error!(%err, "Failed sending request response");
            }
        },
        packet => warn!("unhandled packet {packet:?}"),
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command() {
        create_command("cargo", "/").output().unwrap();
    }
}
