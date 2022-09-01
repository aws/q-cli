use std::ffi::OsStr;
use std::iter::repeat;
use std::path::{
    Path,
    PathBuf,
};
use std::process::Command;
use std::time::SystemTime;

use alacritty_terminal::term::ShellState;
use alacritty_terminal::Term;
use anyhow::Result;
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
};
use fig_proto::figterm::{
    self,
    FigtermRequestMessage,
    FigtermResponseMessage,
};
use fig_proto::secure::{
    clientbound,
    hostbound,
    Clientbound,
    Hostbound,
};
use flume::Sender;
use tracing::{
    debug,
    error,
    trace,
    warn,
};

use crate::event_handler::EventHandler;
use crate::interceptor::KeyInterceptor;
use crate::pty::AsyncMasterPty;
use crate::{
    shell_state_to_context,
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

fn working_directory(path: Option<&str>, shell_state: &ShellState) -> Option<PathBuf> {
    let map_dir = |path: PathBuf| {
        if path.exists() { Some(path) } else { None }
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
}

fn create_command(executable: impl AsRef<OsStr>, working_directory: Option<impl AsRef<Path>>) -> Command {
    let mut cmd = Command::new(executable);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(windows::Win32::System::Threading::DETACHED_PROCESS.0);
    }

    if let Some(working_directory) = working_directory {
        cmd.current_dir(working_directory);
    } else if let Ok(working_directory) = std::env::current_dir() {
        cmd.current_dir(working_directory);
    }

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

    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            if let Some(value) = SHELL_ENVIRONMENT_VARIABLES
                .lock()
                .iter()
                .find_map(|EnvironmentVariable { key, value }| if key == "PATH" { value.as_ref() } else { None })
            {
                cmd.env("PATH", value);
            }
        } else {
            cmd.envs(
                (*SHELL_ENVIRONMENT_VARIABLES.lock())
                    .clone()
                    .into_iter()
                    .filter_map(|EnvironmentVariable { key, value }| value.map(|value| (key, value))),
            );
        }
    }

    cmd.envs([
        ("PROCESS_LAUNCHED_BY_FIG", "1"),
        ("HISTFILE", ""),
        ("HISTCONTROL", "ignoreboth"),
        ("TERM", "xterm-256color"),
    ]);

    cmd
}

/// Process the inner figterm request enum, shared between local and secure
pub async fn process_figterm_request(
    figterm_request: FigtermRequest,
    term: &Term<EventHandler>,
    pty_master: &mut Box<dyn AsyncMasterPty + Send + Sync>,
    key_interceptor: &mut KeyInterceptor,
) -> Result<Option<FigtermResponse>> {
    match figterm_request {
        FigtermRequest::InsertText(request) => {
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
                    let expected = format!("{}{}", buffer, text_to_insert);
                    trace!("lock set, expected buffer: {:?}", expected);
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
                Some(InterceptCommand::SetInterceptAll(_)) => {
                    debug!("Set intercept all");
                    key_interceptor.set_intercept_all(true);
                },
                Some(InterceptCommand::ClearIntercept(_)) => {
                    debug!("Clear intercept");
                    key_interceptor.set_intercept_all(false);
                },
                Some(InterceptCommand::SetFigjsIntercepts(SetFigjsIntercepts {
                    intercept_bound_keystrokes,
                    intercept_global_keystrokes,
                    actions,
                })) => {
                    key_interceptor.set_intercept_all(intercept_global_keystrokes);
                    key_interceptor.set_intercept_bind(intercept_bound_keystrokes);
                    key_interceptor.set_actions(&actions);
                },
                None => {},
            };
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
        FigtermRequest::SetBuffer(_) => todo!(),
        FigtermRequest::UpdateShellContext(request) => {
            if request.update_environment_variables {
                *SHELL_ENVIRONMENT_VARIABLES.lock() = request.environment_variables;
            }
            Ok(None)
        },
    }
}

/// Process a figterm request message
pub async fn process_figterm_message(
    figterm_request_message: FigtermRequestMessage,
    response_tx: Sender<FigtermResponseMessage>,
    term: &Term<EventHandler>,
    pty_master: &mut Box<dyn AsyncMasterPty + Send + Sync>,
    key_interceptor: &mut KeyInterceptor,
) -> Result<()> {
    match figterm_request_message.request {
        Some(request) => match process_figterm_request(request, term, pty_master, key_interceptor).await {
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
    response_tx: Sender<Hostbound>,
    term: &Term<EventHandler>,
    pty_master: &mut Box<dyn AsyncMasterPty + Send + Sync>,
    key_interceptor: &mut KeyInterceptor,
) -> Result<()> {
    use clientbound::request::Request;
    use hostbound::response::Response;

    if let Some(clientbound::Packet::Request(request)) = clientbound_message.packet {
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
                    process_figterm_request(FigtermRequest::InsertText(request), term, pty_master, key_interceptor)
                        .await?,
                    nonce,
                    &response_tx,
                )
                .await;
            },
            Some(Request::Intercept(request)) => {
                send_figterm_response_hostbound(
                    process_figterm_request(FigtermRequest::Intercept(request), term, pty_master, key_interceptor)
                        .await?,
                    nonce,
                    &response_tx,
                )
                .await;
            },
            Some(Request::Diagnostics(request)) => {
                send_figterm_response_hostbound(
                    process_figterm_request(FigtermRequest::Diagnostics(request), term, pty_master, key_interceptor)
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
                    match cmd.output() {
                        Ok(output) => {
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
    }

    Ok(())
}
