mod hooks;

use std::sync::atomic::{
    AtomicU64,
    Ordering,
};
use std::sync::Arc;

use anyhow::Result;
use fig_proto::fig::{
    PseudoterminalExecuteRequest,
    RunProcessRequest,
};
use fig_proto::figterm::{
    intercept_command,
    InsertTextCommand,
    InterceptCommand,
    SetBufferCommand,
};
use fig_proto::local::Empty;
use fig_proto::secure::clientbound::request::Request;
use fig_proto::secure::clientbound::{
    self,
    HandshakeResponse,
};
use fig_proto::secure::{
    hostbound,
    Clientbound,
    Hostbound,
};
use fig_util::directories;
use hashbrown::HashMap;
use time::OffsetDateTime;
use tokio::net::{
    UnixListener,
    UnixStream,
};
use tokio::select;
use tokio::sync::{
    oneshot,
    Notify,
};
use tokio::time::{
    Duration,
    Instant,
};
use tracing::{
    debug,
    error,
    info,
    trace,
    warn,
};

use crate::figterm::{
    EditBuffer,
    FigtermCommand,
    FigtermSession,
    FigtermSessionId,
    FigtermState,
};
use crate::notification::NotificationsState;
use crate::EventLoopProxy;

pub async fn start_secure_ipc(
    figterm_state: Arc<FigtermState>,
    notifications_state: Arc<NotificationsState>,
    proxy: EventLoopProxy,
) -> Result<()> {
    let socket_path = directories::secure_socket_path()?;
    if let Some(parent) = socket_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).expect("Failed creating socket path");
        }
    }

    tokio::fs::remove_file(&socket_path).await.ok();

    let listener = UnixListener::bind(socket_path)?;

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_secure_ipc(
            stream,
            figterm_state.clone(),
            notifications_state.clone(),
            proxy.clone(),
        ));
    }

    Ok(())
}

async fn handle_secure_ipc(
    stream: UnixStream,
    figterm_state: Arc<FigtermState>,
    notifications_state: Arc<NotificationsState>,
    proxy: EventLoopProxy,
) {
    let (mut reader, writer) = tokio::io::split(stream);
    let (clientbound_tx, clientbound_rx) = flume::unbounded();

    let (stop_pings_tx, stop_pings_rx) = oneshot::channel();

    let bad_connection = Arc::new(Notify::new());

    let outgoing_task = tokio::spawn(handle_outgoing(writer, clientbound_rx, bad_connection.clone()));
    let ping_task = tokio::spawn(send_pings(clientbound_tx.clone(), stop_pings_rx));

    let mut session_id: Option<FigtermSessionId> = None;

    while let Some(message) = fig_ipc::recv_message::<Hostbound, _>(&mut reader)
        .await
        .unwrap_or_else(|err| {
            if !err.is_disconnect() {
                error!(%err, "Failed receiving secure message");
            }
            None
        })
    {
        trace!(?message, "Received secure message");
        if let Some(response) = match message.packet {
            Some(hostbound::Packet::Handshake(handshake)) => {
                if session_id.is_some() {
                    // maybe they missed our response, but they should've been listening harder
                    Some(clientbound::Packet::HandshakeResponse(HandshakeResponse {
                        success: false,
                    }))
                } else {
                    let id = FigtermSessionId(handshake.id);
                    match figterm_state.sessions.get_mut(&id) {
                        Some(mut session) => {
                            if session.secret == handshake.secret {
                                session_id = Some(id);
                                session.writer = Some(clientbound_tx.clone());
                                session.dead_since = None;
                                debug!("Client auth accepted because of secret match");
                                Some(clientbound::Packet::HandshakeResponse(HandshakeResponse {
                                    success: true,
                                }))
                            } else {
                                debug!("Client auth rejected because of secret mismatch");
                                Some(clientbound::Packet::HandshakeResponse(HandshakeResponse {
                                    success: false,
                                }))
                            }
                        },
                        None => {
                            session_id = Some(id.clone());
                            let (command_tx, command_rx) = flume::unbounded();
                            tokio::spawn(handle_commands(command_rx, figterm_state.clone(), id.clone()));
                            figterm_state.insert(id, FigtermSession {
                                secret: handshake.secret,
                                sender: command_tx,
                                writer: Some(clientbound_tx.clone()),
                                dead_since: None,
                                last_receive: Instant::now(),
                                edit_buffer: EditBuffer {
                                    text: "".to_string(),
                                    cursor: 0,
                                },
                                context: None,
                                terminal_cursor_coordinates: None,
                                current_session_metrics: None,
                                response_map: HashMap::new(),
                                nonce_counter: Arc::new(AtomicU64::new(0)),
                            });
                            debug!("Client auth accepted because of new id");
                            Some(clientbound::Packet::HandshakeResponse(HandshakeResponse {
                                success: true,
                            }))
                        },
                    }
                }
            },
            Some(hostbound::Packet::Hook(hostbound::Hook { hook: Some(hook) })) => {
                if let Some(session_id) = &session_id {
                    if let Err(err) = match hook {
                        hostbound::hook::Hook::EditBuffer(edit_buffer) => {
                            hooks::edit_buffer(
                                &edit_buffer,
                                session_id,
                                figterm_state.clone(),
                                &notifications_state,
                                &proxy,
                            )
                            .await
                        },
                        hostbound::hook::Hook::Prompt(prompt) => {
                            hooks::prompt(&prompt, &notifications_state, &proxy).await
                        },
                        hostbound::hook::Hook::PreExec(pre_exec) => {
                            hooks::pre_exec(&pre_exec, &notifications_state, &proxy).await
                        },
                        hostbound::hook::Hook::InterceptedKey(intercepted_key) => {
                            hooks::intercepted_key(intercepted_key, &notifications_state, &proxy).await
                        },
                    } {
                        error!(%err, "Failed processing hook")
                    }
                    None
                } else {
                    // apparently they didn't get the memo
                    debug!("Client tried to send secure hook without auth");
                    Some(clientbound::Packet::HandshakeResponse(HandshakeResponse {
                        success: false,
                    }))
                }
            },
            Some(hostbound::Packet::Response(hostbound::Response {
                nonce,
                response: Some(response),
            })) => {
                if let Some(nonce) = nonce {
                    session_id
                        .as_ref()
                        .and_then(|session_id| {
                            figterm_state.with_mut(session_id.clone(), |session| session.response_map.remove(&nonce))
                        })
                        .flatten()
                        .map(|channel| channel.send(response));
                }
                None
            },
            _ => {
                warn!("Received invalid secure packet");
                None
            },
        } {
            let _ = clientbound_tx.send(Clientbound { packet: Some(response) });
        }
    }

    let _ = stop_pings_tx.send(());
    drop(clientbound_tx);

    if let Some(session_id) = &session_id {
        if let Some(mut session) = figterm_state.sessions.get_mut(session_id) {
            session.writer = None;
            session.dead_since = Some(Instant::now());
        }
    }

    if let Err(err) = ping_task.await {
        error!(%err, "Secure ping task join error");
    }

    if let Err(err) = outgoing_task.await {
        error!(%err, "Secure outgoing task join error");
    }

    info!("Disconnect from {session_id:?}");
}

async fn handle_outgoing(
    mut writer: tokio::io::WriteHalf<UnixStream>,
    outgoing: flume::Receiver<Clientbound>,
    bad_connection: Arc<Notify>,
) {
    while let Ok(message) = outgoing.recv_async().await {
        trace!(?message, "Sending secure message");
        if let Err(err) = fig_ipc::send_message(&mut writer, message).await {
            error!(%err, "Secure outgoing task send error");
            bad_connection.notify_one();
            return;
        }
    }
}

async fn handle_commands(
    incoming: flume::Receiver<FigtermCommand>,
    figterm_state: Arc<FigtermState>,
    session_id: FigtermSessionId,
) -> Option<()> {
    while let Ok(command) = incoming.recv_async().await {
        let (request, nonce_channel) = match command {
            FigtermCommand::InterceptDefault => (
                Request::Intercept(InterceptCommand {
                    intercept_command: Some(intercept_command::InterceptCommand::SetInterceptAll(
                        intercept_command::SetInterceptAll {},
                    )),
                }),
                None,
            ),
            FigtermCommand::InterceptClear => (
                Request::Intercept(InterceptCommand {
                    intercept_command: Some(intercept_command::InterceptCommand::ClearIntercept(
                        intercept_command::ClearIntercept {},
                    )),
                }),
                None,
            ),
            FigtermCommand::InterceptFigJs {
                intercept_bound_keystrokes,
                intercept_global_keystrokes,
                actions,
            } => (
                Request::Intercept(InterceptCommand {
                    intercept_command: Some(intercept_command::InterceptCommand::SetFigjsIntercepts(
                        intercept_command::SetFigjsIntercepts {
                            intercept_bound_keystrokes,
                            intercept_global_keystrokes,
                            actions,
                        },
                    )),
                }),
                None,
            ),
            FigtermCommand::InsertText {
                insertion,
                deletion,
                offset,
                immediate,
                insertion_buffer,
            } => (
                Request::InsertText(InsertTextCommand {
                    insertion,
                    deletion: deletion.map(|x| x as u64),
                    offset,
                    immediate,
                    insertion_buffer,
                }),
                None,
            ),
            FigtermCommand::SetBuffer { text, cursor_position } => {
                (Request::SetBuffer(SetBufferCommand { text, cursor_position }), None)
            },
            FigtermCommand::RunProcess {
                channel,
                executable,
                arguments,
                working_directory,
                env,
            } => (
                Request::RunProcess(RunProcessRequest {
                    executable,
                    arguments,
                    working_directory,
                    env,
                }),
                Some(channel),
            ),
            FigtermCommand::PseudoterminalExecute {
                channel,
                command,
                working_directory,
                background_job,
                is_pipelined,
                env,
            } => (
                Request::PseudoterminalExecute(PseudoterminalExecuteRequest {
                    command,
                    working_directory,
                    background_job,
                    is_pipelined,
                    env,
                }),
                Some(channel),
            ),
        };

        let nonce = if let Some(channel) = nonce_channel {
            Some(figterm_state.with_mut(session_id.clone(), |session| {
                let nonce = session.nonce_counter.fetch_add(1, Ordering::Relaxed);
                session.response_map.insert(nonce, channel);
                nonce
            })?)
        } else {
            None
        };

        let is_insert_request = matches!(request, Request::InsertText(_));
        figterm_state.with_mut(session_id.clone(), |session| {
            if let Some(writer) = &session.writer {
                if writer
                    .try_send(Clientbound {
                        packet: Some(clientbound::Packet::Request(clientbound::Request {
                            request: Some(request),
                            nonce,
                        })),
                    })
                    .is_ok()
                {
                    if is_insert_request {
                        if let Some(ref mut metrics) = session.current_session_metrics {
                            metrics.num_insertions += 1;
                            metrics.end_time =
                                OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
                        }
                    }
                    session.last_receive = Instant::now();
                };
            }
        })?;
    }

    None
}

async fn send_pings(outgoing: flume::Sender<Clientbound>, mut stop_pings: oneshot::Receiver<()>) {
    let mut interval = tokio::time::interval(Duration::from_secs(5));

    loop {
        select! {
            _ = interval.tick() => {
                let _ = outgoing.try_send(Clientbound {
                    packet: Some(clientbound::Packet::Ping(Empty {})),
                });
            }
            _ = &mut stop_pings => break
        }
    }
}
