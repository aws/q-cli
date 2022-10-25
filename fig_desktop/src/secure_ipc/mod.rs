mod hooks;

use std::sync::atomic::{
    AtomicU64,
    Ordering,
};
use std::sync::Arc;

use anyhow::Result;
use fig_ipc::{
    BufferedReader,
    RecvMessage,
    SendMessage,
};
use fig_proto::figterm::{
    intercept_request,
    InsertTextRequest,
    InterceptRequest,
    SetBufferRequest,
};
use fig_proto::local::ShellContext;
use fig_proto::secure::clientbound::request::Request;
use fig_proto::secure::clientbound::{
    self,
    HandshakeResponse,
    PseudoterminalExecuteRequest,
    RunProcessRequest,
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
use tokio::sync::Notify;
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
use crate::webview::notification::WebviewNotificationsState;
use crate::EventLoopProxy;

pub async fn start_secure_ipc(
    figterm_state: Arc<FigtermState>,
    notifications_state: Arc<WebviewNotificationsState>,
    proxy: EventLoopProxy,
) -> Result<()> {
    let socket_path = directories::secure_socket_path()?;
    if let Some(parent) = socket_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).expect("Failed creating socket path");
        }
    }

    #[cfg(unix)]
    if let Err(err) = fig_ipc::util::set_sockets_dir_permissions() {
        error!(%err, "Failed to set permissions on sockets directory");
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
    notifications_state: Arc<WebviewNotificationsState>,
    proxy: EventLoopProxy,
) {
    let (reader, writer) = tokio::io::split(stream);
    let (clientbound_tx, clientbound_rx) = flume::unbounded();

    let bad_connection = Arc::new(Notify::new());

    let (on_close_tx, mut on_close_rx) = tokio::sync::broadcast::channel(1);

    let outgoing_task = tokio::spawn(handle_outgoing(
        writer,
        clientbound_rx,
        bad_connection.clone(),
        on_close_tx.subscribe(),
    ));

    let ping_task = tokio::spawn(send_pings(clientbound_tx.clone(), on_close_tx.subscribe()));

    let mut session_id: Option<FigtermSessionId> = None;

    let mut reader = BufferedReader::new(reader);
    loop {
        tokio::select! {
            _ = on_close_rx.recv() => {
                debug!("Connection closed");
                break;
            }
            message = reader.recv_message::<Hostbound>() => match message {
                Ok(Some(message)) => {
                    trace!(?message, "Received secure message");
                    if let Some(response) = match message.packet {
                        Some(hostbound::Packet::Handshake(handshake)) => {
                            if session_id.is_some() {
                                // maybe they missed our response, but they should've been listening harder
                                Some(clientbound::Packet::HandshakeResponse(HandshakeResponse {
                                    success: false,
                                }))
                            } else {
                                let id = FigtermSessionId(handshake.id.clone());

                                if let Some(success) = figterm_state.with_update(id.clone(), |session| {
                                    if session.secret == handshake.secret {
                                        session_id = Some(id);
                                        session.writer = Some(clientbound_tx.clone());
                                        session.dead_since = None;
                                        session.on_close_tx = on_close_tx.clone();
                                        debug!(
                                            "Client auth for {} accepted because of secret match ({} = {})",
                                            handshake.id, session.secret, handshake.secret
                                        );
                                        true
                                    } else {
                                        debug!(
                                            "Client auth for {} rejected because of secret mismatch ({} =/= {})",
                                            handshake.id, session.secret, handshake.secret
                                        );
                                        false
                                    }
                                }) {
                                    Some(clientbound::Packet::HandshakeResponse(HandshakeResponse { success }))
                                } else {
                                    let id = FigtermSessionId(handshake.id.clone());
                                    session_id = Some(id.clone());
                                    let (command_tx, command_rx) = flume::unbounded();
                                    tokio::spawn(handle_commands(command_rx, figterm_state.clone(), id.clone()));
                                    figterm_state.insert(FigtermSession {
                                        id,
                                        secret: handshake.secret.clone(),
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
                                        on_close_tx: on_close_tx.clone(),
                                    });
                                    debug!(
                                        "Client auth for {} accepted because of new id with secret {}",
                                        handshake.id, handshake.secret
                                    );
                                    Some(clientbound::Packet::HandshakeResponse(HandshakeResponse {
                                        success: true,
                                    }))
                                }
                            }
                        },
                        Some(hostbound::Packet::Hook(hostbound::Hook { hook: Some(hook) })) => {
                            if let Some(session_id) = &session_id {
                                let sanatize_fn = get_sanatize_fn(session_id.0.clone());
                                if let Err(err) = match hook {
                                    hostbound::hook::Hook::EditBuffer(mut edit_buffer) => {
                                        sanatize_fn(&mut edit_buffer.context);
                                        hooks::edit_buffer(
                                            &edit_buffer,
                                            session_id,
                                            figterm_state.clone(),
                                            &notifications_state,
                                            &proxy,
                                        )
                                        .await
                                    },
                                    hostbound::hook::Hook::Prompt(mut prompt) => {
                                        sanatize_fn(&mut prompt.context);
                                        hooks::prompt(&prompt, session_id, &figterm_state, &notifications_state, &proxy).await
                                    },
                                    hostbound::hook::Hook::PreExec(mut pre_exec) => {
                                        sanatize_fn(&mut pre_exec.context);
                                        hooks::pre_exec(&pre_exec, session_id, &figterm_state, &notifications_state, &proxy).await
                                    },
                                    hostbound::hook::Hook::InterceptedKey(mut intercepted_key) => {
                                        sanatize_fn(&mut intercepted_key.context);
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
                                        figterm_state.with(session_id, |session| session.response_map.remove(&nonce))
                                    })
                                    .flatten()
                                    .map(|channel| channel.send(response));
                            }
                            None
                        },
                        Some(hostbound::Packet::Pong(())) => {
                            trace!(?session_id, "Received pong");
                            if let Some(session_id) = &session_id {
                                figterm_state.with(session_id, |session| {
                                    session.last_receive = Instant::now();
                                });
                            }
                            None
                        },
                        Some(hostbound::Packet::Hook(hostbound::Hook { hook: None }))
                            | Some(hostbound::Packet::Response(hostbound::Response { response: None, .. }))
                            | None => {
                            warn!("Received invalid secure packet");
                            None
                        }
                    } {
                        let _ = clientbound_tx.send(Clientbound { packet: Some(response) });
                    }
                }
                Ok(None) => {
                    debug!("Figterm connection closed");
                    break;
                }
                Err(err) => {
                    if !err.is_disconnect() {
                        warn!(%err, "Failed receiving secure message");
                    }
                    break;
                }
            }
        }
    }

    let _ = on_close_tx.send(());
    drop(clientbound_tx);

    if let Some(session_id) = &session_id {
        figterm_state.with_update(session_id.clone(), |session| {
            session.writer = None;
            session.dead_since = Some(Instant::now());
        });
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
    mut on_close_rx: tokio::sync::broadcast::Receiver<()>,
) {
    loop {
        tokio::select! {
            _ = on_close_rx.recv() => {
                debug!("Secure outgoing task exiting");
                break;
            },
            message = outgoing.recv_async() => {
                if let Ok(message) = message {
                    debug!(?message, "Sending secure message");
                    if let Err(err) = writer.send_message(message).await {
                        error!(%err, "Secure outgoing task send error");
                        bad_connection.notify_one();
                        return;
                    }
                } else {
                    debug!("Secure outgoing task exiting");
                    break;
                }
            }
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
                Request::Intercept(InterceptRequest {
                    intercept_command: Some(intercept_request::InterceptCommand::SetInterceptAll(())),
                }),
                None,
            ),
            FigtermCommand::InterceptClear => (
                Request::Intercept(InterceptRequest {
                    intercept_command: Some(intercept_request::InterceptCommand::ClearIntercept(())),
                }),
                None,
            ),
            FigtermCommand::InterceptFigJs {
                intercept_bound_keystrokes,
                intercept_global_keystrokes,
                actions,
            } => (
                Request::Intercept(InterceptRequest {
                    intercept_command: Some(intercept_request::InterceptCommand::SetFigjsIntercepts(
                        intercept_request::SetFigjsIntercepts {
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
                insert_during_command,
            } => (
                Request::InsertText(InsertTextRequest {
                    insertion,
                    deletion: deletion.map(|x| x as u64),
                    offset,
                    immediate,
                    insertion_buffer,
                    insert_during_command,
                }),
                None,
            ),
            FigtermCommand::SetBuffer { text, cursor_position } => {
                (Request::SetBuffer(SetBufferRequest { text, cursor_position }), None)
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
            Some(figterm_state.with(&session_id, |session| {
                let nonce = session.nonce_counter.fetch_add(1, Ordering::Relaxed);
                session.response_map.insert(nonce, channel);
                nonce
            })?)
        } else {
            None
        };

        let is_insert_request = matches!(request, Request::InsertText(_));
        figterm_state.with(&session_id, |session| {
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

async fn send_pings(outgoing: flume::Sender<Clientbound>, mut on_close_rx: tokio::sync::broadcast::Receiver<()>) {
    let mut interval = tokio::time::interval(Duration::from_secs(5));

    loop {
        select! {
            _ = interval.tick() => {
                let _ = outgoing.try_send(Clientbound {
                    packet: Some(clientbound::Packet::Ping(())),
                });
            }
            _ = on_close_rx.recv() => break
        }
    }
}

// This has to be used to sanitize as a hook can contain an invalid session_id and it must
// be sanitized before being sent to any consumers
fn get_sanatize_fn(session_id: String) -> impl FnOnce(&mut Option<ShellContext>) {
    |context| {
        if let Some(context) = context {
            context.session_id = Some(session_id);
        }
    }
}
