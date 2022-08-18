//! Utiities for IPC with Mac App

use std::fmt::Display;
use std::iter::repeat;
use std::time::{
    Duration,
    SystemTime,
};

use alacritty_terminal::Term;
use anyhow::Result;
use fig_proto::figterm::{
    self,
    figterm_message,
    intercept_command,
    FigtermMessage,
    FigtermResponse,
};
use fig_proto::FigProtobufEncodable;
use fig_util::directories;
use flume::{
    unbounded,
    Receiver,
    Sender,
};
use system_socket::SystemListener;
use tokio::io::AsyncWriteExt;
use tracing::{
    debug,
    error,
    trace,
};

use crate::event_handler::EventHandler;
use crate::interceptor::KeyInterceptor;
use crate::pty::AsyncMasterPty;
use crate::{
    shell_state_to_context,
    MainLoopEvent,
    EXECUTE_ON_NEW_CMD,
    EXPECTED_BUFFER,
    INSERTION_LOCKED_AT,
    INSERT_ON_NEW_CMD,
};

pub async fn create_socket_listen(session_id: impl Display) -> Result<SystemListener> {
    let socket_path = directories::figterm_socket_path(session_id)?;
    tokio::fs::remove_file(&socket_path).await.ok();
    Ok(SystemListener::bind(&socket_path)?)
}

pub async fn remove_socket(session_id: impl Display) -> Result<()> {
    let socket_path = directories::figterm_socket_path(session_id)?;
    tokio::fs::remove_file(&socket_path).await.ok();
    Ok(())
}

/// Spawn a thread to send events to Fig desktop app
pub async fn spawn_outgoing_sender(
    main_loop_sender: Sender<MainLoopEvent>,
) -> Result<Sender<fig_proto::local::LocalMessage>> {
    trace!("Spawning outgoing sender");
    let (outgoing_tx, outgoing_rx) = unbounded::<fig_proto::local::LocalMessage>();
    let socket = directories::fig_socket_path()?;

    tokio::spawn(async move {
        let unlock_interceptor = || {
            main_loop_sender
                .send(MainLoopEvent::Insert {
                    insert: Vec::new(),
                    unlock: true,
                })
                .unwrap();
        };

        while let Ok(message) = outgoing_rx.recv_async().await {
            match fig_ipc::connect_timeout(&socket, Duration::from_secs(1)).await {
                Ok(mut unix_stream) => match fig_ipc::send_message(&mut unix_stream, message).await {
                    Ok(()) => {
                        if let Err(err) = unix_stream.flush().await {
                            error!(%err, "Failed to flush socket");
                            unlock_interceptor();
                        }
                    },
                    Err(err) => {
                        error!(%err, "Failed to send message");
                        unlock_interceptor();
                    },
                },
                Err(err) => {
                    error!(%err, "Error connecting to socket");
                    unlock_interceptor();
                },
            }
        }
    });

    Ok(outgoing_tx)
}

pub async fn spawn_incoming_receiver(
    session_id: impl Display,
) -> Result<Receiver<(FigtermMessage, Sender<FigtermResponse>)>> {
    trace!("Spawning incoming receiver");

    let (incoming_tx, incoming_rx) = unbounded();

    let socket_listener = create_socket_listen(session_id).await?;
    tokio::spawn(async move {
        loop {
            if let Ok(stream) = socket_listener.accept().await {
                trace!("Accepted connection.");
                let incoming_tx = incoming_tx.clone();

                let (mut read_half, mut write_half) = tokio::io::split(stream);
                let (response_tx, response_rx) = unbounded::<FigtermResponse>();

                tokio::spawn(async move {
                    let mut rx_thread = tokio::spawn(async move {
                        loop {
                            match fig_ipc::recv_message::<FigtermMessage, _>(&mut read_half).await {
                                Ok(Some(message)) => {
                                    debug!("Received message: {message:?}");
                                    incoming_tx
                                        .clone()
                                        .send_async((message, response_tx.clone()))
                                        .await
                                        .unwrap();
                                },
                                Ok(None) => {
                                    debug!("Received EOF");
                                    break;
                                },
                                Err(err) => {
                                    error!("Error receiving message: {err}");
                                    break;
                                },
                            }
                        }
                    });

                    loop {
                        tokio::select! {
                            // Break once the rx_thread quits
                            _ = &mut rx_thread => break,
                            res = response_rx.recv_async() => {
                                match res {
                                    Ok(message) => {
                                        match message.encode_fig_protobuf() {
                                            Ok(protobuf) => {
                                                if let Err(err) = write_half.write_all(&protobuf).await {
                                                    error!("Failed to send response: {err}");
                                                    break;
                                                }
                                            },
                                            Err(err) => {
                                                error!("Failed to encode protobuf: {err}")
                                            }
                                        }
                                    }
                                    Err(_) => break,
                                }
                            }
                        }
                    }
                });
            }
        }
    });

    Ok(incoming_rx)
}

pub async fn process_figterm_message(
    figterm_message: FigtermMessage,
    response_tx: Sender<FigtermResponse>,
    term: &Term<EventHandler>,
    pty_master: &mut Box<dyn AsyncMasterPty + Send + Sync>,
    key_interceptor: &mut KeyInterceptor,
) -> Result<()> {
    match figterm_message.command {
        Some(figterm_message::Command::InsertTextCommand(command)) => {
            let current_buffer = term.get_current_buffer().map(|buff| (buff.buffer, buff.cursor_idx));
            let mut insertion_string = String::new();
            if let Some((buffer, Some(position))) = current_buffer {
                if let Some(ref text_to_insert) = command.insertion {
                    trace!("buffer: {buffer:?}, cursor_position: {position:?}");

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
                    trace!("lock set, expected buffer: {expected:?}");
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
        Some(figterm_message::Command::InterceptCommand(command)) => match command.intercept_command {
            Some(intercept_command::InterceptCommand::SetInterceptAll(_)) => {
                debug!("Set intercept all");
                key_interceptor.set_intercept_all(true);
            },
            Some(intercept_command::InterceptCommand::ClearIntercept(_)) => {
                debug!("Clear intercept");
                key_interceptor.set_intercept_all(false);
            },
            Some(intercept_command::InterceptCommand::SetFigjsIntercepts(intercept_command::SetFigjsIntercepts {
                intercept_bound_keystrokes,
                intercept_global_keystrokes,
                actions,
            })) => {
                key_interceptor.set_intercept_all(intercept_global_keystrokes);
                key_interceptor.set_intercept_bind(intercept_bound_keystrokes);
                key_interceptor.set_actions(&actions);
            },
            None => {},
        },
        Some(figterm_message::Command::DiagnosticsCommand(_command)) => {
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

            if let Err(err) = response_tx
                .send_async(FigtermResponse {
                    response: Some(figterm::figterm_response::Response::DiagnosticsResponse(
                        figterm::DiagnosticsResponse {
                            shell_context: Some(shell_state_to_context(term.shell_state())),
                            fish_suggestion_style: term.shell_state().fish_suggestion_color.as_ref().map(map_style),
                            zsh_autosuggestion_style: term
                                .shell_state()
                                .zsh_autosuggestion_color
                                .as_ref()
                                .map(map_style),
                            edit_buffer,
                            cursor_position,
                        },
                    )),
                })
                .await
            {
                error!("Failed to send response: {err}");
            }
        },
        Some(figterm_message::Command::InsertOnNewCmdCommand(figterm::InsertOnNewCmdCommand { text, execute })) => {
            *INSERT_ON_NEW_CMD.lock() = Some(text);
            *EXECUTE_ON_NEW_CMD.lock() = execute;
        },
        Some(figterm_message::Command::SetBufferCommand(_)) => {},
        None => {},
    }

    Ok(())
}
