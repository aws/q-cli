use std::ffi::OsStr;
use std::iter::repeat;
use std::process::Command;
use std::time::SystemTime;

use alacritty_terminal::Term;
use anyhow::Result;
use fig_proto::fig::{
    EnvironmentVariable,
    PseudoterminalExecuteResponse,
    RunProcessResponse,
};
use fig_proto::figterm;
use fig_proto::figterm::intercept_command::{
    self,
    SetFigjsIntercepts,
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

fn create_command(executable: impl AsRef<OsStr>, working_directory: Option<String>) -> Command {
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

    cmd.envs([
        ("PROCESS_LAUNCHED_BY_FIG", "1"),
        ("FIG_NO_RAN_COMMAND", "1"),
        ("HISTFILE", ""),
        ("HISTCONTROL", "ignoreboth"),
        ("TERM", "xterm-256color"),
    ]);

    cmd
}

pub async fn process_figterm_message(
    figterm_message: Clientbound,
    response_tx: Sender<Hostbound>,
    term: &Term<EventHandler>,
    pty_master: &mut Box<dyn AsyncMasterPty + Send + Sync>,
    key_interceptor: &mut KeyInterceptor,
) -> Result<()> {
    use clientbound::request::Request;
    use hostbound::response::Response;

    if let Some(clientbound::Packet::Request(request)) = figterm_message.packet {
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
            Some(Request::InsertText(command)) => {
                let current_buffer = term.get_current_buffer().map(|buff| (buff.buffer, buff.cursor_idx));
                let mut insertion_string = String::new();
                if let Some((buffer, Some(position))) = current_buffer {
                    if let Some(ref text_to_insert) = command.insertion {
                        trace!("buffer: {:?}, cursor_position: {:?}", buffer, position);

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
                    if let Some(ref insertion_buffer) = command.insertion_buffer {
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
                insertion_string.push_str(&command.to_term_string());
                pty_master.write(insertion_string.as_bytes()).await?;
            },
            Some(Request::Intercept(command)) => {
                match command.intercept_command {
                    Some(intercept_command::InterceptCommand::SetInterceptAll(_)) => {
                        debug!("Set intercept all");
                        key_interceptor.set_intercept_all(true);
                    },
                    Some(intercept_command::InterceptCommand::ClearIntercept(_)) => {
                        debug!("Clear intercept");
                        key_interceptor.set_intercept_all(false);
                    },
                    Some(intercept_command::InterceptCommand::SetFigjsIntercepts(SetFigjsIntercepts {
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
            },
            Some(Request::Diagnostics(_)) => {
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

                let response = make_response(Response::Diagnostics(figterm::DiagnosticsResponse {
                    shell_context: Some(shell_state_to_context(term.shell_state())),
                    fish_suggestion_style: term.shell_state().fish_suggestion_color.as_ref().map(map_style),
                    zsh_autosuggestion_style: term.shell_state().zsh_autosuggestion_color.as_ref().map(map_style),
                    edit_buffer,
                    cursor_position,
                }));

                if let Err(err) = response_tx.send_async(response).await {
                    error!("failed sending request response: {err}");
                }
            },
            Some(Request::InsertOnNewCmd(request)) => {
                *INSERT_ON_NEW_CMD.lock() = Some(request.text);
                *EXECUTE_ON_NEW_CMD.lock() = request.execute;
            },
            Some(Request::RunProcess(request)) => {
                // TODO(sean) we can infer shell as above for execute if no executable is provided.
                let mut cmd = create_command(&request.executable, request.working_directory);

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
                                error!("failed sending request response: {err}");
                            }
                        },
                        Err(err) => {
                            warn!("Failed running command {}: {}", request.executable, err);
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

                let mut cmd = create_command(&default_command_shell, request.working_directory);
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
                            warn!("Failed running command {}: {}", request.command, err);
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
                                error!("failed sending request response: {err}");
                            }
                        },
                    }
                });
            },
            _ => {
                warn!("unhandled request {request:?}");
            },
        }
    }

    Ok(())
}
