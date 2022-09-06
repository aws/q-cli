//! Utiities for IPC with Tauri App

use std::io;
use std::pin::Pin;
use std::task::{
    Context,
    Poll,
};
use std::time::Duration;

use anyhow::Result;
use fig_ipc::{
    BufferedReader,
    RecvMessage,
    SendMessage,
};
use fig_proto::figterm::{
    FigtermRequestMessage,
    FigtermResponseMessage,
};
use fig_proto::secure::hostbound::Handshake;
use fig_proto::secure::{
    clientbound,
    hostbound,
    Clientbound,
    Hostbound,
};
use fig_proto::FigProtobufEncodable;
use fig_util::{
    directories,
    gen_hex_string,
};
use flume::{
    unbounded,
    Receiver,
    Sender,
};
use pin_project::pin_project;
use tokio::io::{
    AsyncRead,
    AsyncWrite,
    AsyncWriteExt,
    ReadBuf,
};
use tokio::join;
use tokio::process::{
    ChildStdin,
    ChildStdout,
};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::interval;
use tracing::{
    debug,
    error,
    info,
    trace,
};

use crate::MainLoopEvent;

#[allow(dead_code)]
#[pin_project(project = MessageSourceProj)]
enum MessageSource {
    UnixStream(#[pin] tokio::io::ReadHalf<tokio::net::UnixStream>),
    ChildStdout(#[pin] ChildStdout),
}

impl AsyncRead for MessageSource {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        match self.project() {
            MessageSourceProj::UnixStream(stream) => stream.poll_read(cx, buf),
            MessageSourceProj::ChildStdout(stdout) => stdout.poll_read(cx, buf),
        }
    }
}

#[allow(dead_code)]
#[pin_project(project = MessageSinkProj)]
enum MessageSink {
    UnixStream(#[pin] tokio::io::WriteHalf<tokio::net::UnixStream>),
    ChildStdin(#[pin] ChildStdin),
}

impl AsyncWrite for MessageSink {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, io::Error>> {
        match self.project() {
            MessageSinkProj::UnixStream(stream) => stream.poll_write(cx, buf),
            MessageSinkProj::ChildStdin(stdin) => stdin.poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match self.project() {
            MessageSinkProj::UnixStream(stream) => stream.poll_flush(cx),
            MessageSinkProj::ChildStdin(stdin) => stdin.poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match self.project() {
            MessageSinkProj::UnixStream(stream) => stream.poll_shutdown(cx),
            MessageSinkProj::ChildStdin(stdin) => stdin.poll_shutdown(cx),
        }
    }
}

async fn get_forwarded_stream() -> Result<(MessageSource, MessageSink, Option<JoinHandle<()>>)> {
    #[cfg(target_os = "linux")]
    if fig_util::system_info::in_wsl() {
        use std::process::Stdio;

        use anyhow::Context as AnyhowContext;

        let mut child = tokio::process::Command::new("fig.exe")
            .args(["_", "stream-from-socket"])
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take().context("Failed to open stdin")?;
        let stdout = child.stdout.take().context("Failed to open stdout")?;

        let child_task = tokio::spawn(async move {
            if let Err(e) = child.wait().await {
                error!("Error waiting for child {e:?}");
            }
        });

        return Ok((
            MessageSource::ChildStdout(stdout),
            MessageSink::ChildStdin(stdin),
            Some(child_task),
        ));
    }

    let socket = directories::secure_socket_path()?;
    let stream = fig_ipc::socket_connect_timeout(&socket, Duration::from_secs(5)).await?;
    let (reader, writer) = tokio::io::split(stream);
    Ok((MessageSource::UnixStream(reader), MessageSink::UnixStream(writer), None))
}

/// Spawns a local unix socket for communicating with figterm on a local machine
pub async fn spawn_figterm_ipc(
    session_id: impl std::fmt::Display,
) -> Result<Receiver<(FigtermRequestMessage, Sender<FigtermResponseMessage>)>> {
    trace!("Spawning incoming receiver");

    let (incoming_tx, incoming_rx) = unbounded();

    let socket_path = directories::figterm_socket_path(session_id)?;
    tokio::fs::remove_file(&socket_path).await.ok();
    let socket_listener = tokio::net::UnixListener::bind(&socket_path)?;

    tokio::spawn(async move {
        loop {
            if let Ok((stream, _)) = socket_listener.accept().await {
                let incoming_tx = incoming_tx.clone();

                let (read_half, mut write_half) = tokio::io::split(stream);
                let (response_tx, response_rx) = unbounded::<FigtermResponseMessage>();

                tokio::spawn(async move {
                    let mut read_half = BufferedReader::new(read_half);
                    let mut rx_thread = tokio::spawn(async move {
                        loop {
                            match read_half.recv_message::<FigtermRequestMessage>().await {
                                Ok(Some(message)) => {
                                    // debug!("Received message: {message:?}");
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
                                    Ok(response) => {
                                        match response.encode_fig_protobuf() {
                                            Ok(protobuf) => {
                                                if let Err(err) = write_half.write_all(&protobuf).await {
                                                    error!("Failed to send response: {err}");
                                                    break;
                                                }
                                            },
                                            Err(err) => error!("Failed to encode protobuf: {err}")
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

/// Connects to the desktop app and allows for a secure connection from remote hosts
pub async fn spawn_secure_ipc(
    session_id: String,
    main_loop_sender: Sender<MainLoopEvent>,
) -> Result<(Sender<Hostbound>, Receiver<Clientbound>, oneshot::Sender<()>)> {
    let (stop_ipc_tx, mut stop_ipc_rx) = oneshot::channel::<()>();
    let (outgoing_tx, outgoing_rx) = unbounded::<Hostbound>();
    let (incoming_tx, incoming_rx) = unbounded::<Clientbound>();

    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(5));
        let secret = gen_hex_string();

        loop {
            interval.tick().await;
            tokio::select! {
                _ = &mut stop_ipc_rx => {
                    break;
                }
                res = get_forwarded_stream() => {
                    if let Ok((reader, mut writer, child)) = res {
                        let mut reader = BufferedReader::new(reader);
                        info!("Attempting handshake...");
                        if let Err(err) = writer.send_message(Hostbound {
                            packet: Some(hostbound::Packet::Handshake(Handshake {
                                id: session_id.clone(),
                                secret: secret.clone(),
                            })),
                        })
                        .await
                        {
                            error!("error sending handshake: {err}");
                            continue;
                        }
                        let mut handshake_success = false;
                        info!("Awaiting handshake response...");
                        while let Some(message) = reader.recv_message::<Clientbound>().await.unwrap_or_else(|err| {
                            error!("failed receiving handshake response: {err}");
                            None
                        }) {
                            if let Some(clientbound::Packet::HandshakeResponse(response)) = message.packet {
                                handshake_success = response.success;
                                break;
                            }
                        }
                        if !handshake_success {
                            error!("failed performing handshake");
                            continue;
                        }
                        info!("Handshake succeeded");

                        // send outgoing messages
                        outgoing_rx.drain();
                        let outgoing_rx = outgoing_rx.clone();
                        let main_loop_sender = main_loop_sender.clone();
                        let outgoing_task = tokio::spawn(async move {
                            while let Ok(message) = outgoing_rx.recv_async().await {
                                match writer.send_message(message.clone()).await {
                                    Ok(()) => {
                                        if let Err(err) = writer.flush().await {
                                            error!(%err, "Failed to flush socket");
                                            main_loop_sender
                                                .send(MainLoopEvent::Insert {
                                                    insert: Vec::new(),
                                                    unlock: true,
                                                })
                                                .unwrap();
                                        }
                                    }
                                    Err(err) => {
                                        error!(%err, "Failed to send message");
                                        main_loop_sender
                                            .send(MainLoopEvent::Insert {
                                                insert: Vec::new(),
                                                unlock: true,
                                            })
                                            .unwrap();
                                        let _ = writer.shutdown().await;
                                        break;
                                    }
                                }
                            }
                        });

                        // receive incoming messages
                        let incoming_tx = incoming_tx.clone();
                        let incoming_task = tokio::spawn(async move {
                            while let Some(message) = reader.recv_message().await.unwrap_or_else(|err| {
                                error!("failed receiving message from host: {err}");
                                None
                            }) {
                                if let Err(err) = incoming_tx.send(message) {
                                    error!("no more listeners for incoming messages: {err}");
                                    break;
                                }
                            }
                        });

                        if let Some(child) = child {
                            let _ = join!(outgoing_task, incoming_task, child);
                        } else {
                            let _ = join!(outgoing_task, incoming_task);
                        }
                    }
                }
            }
        }
    });

    Ok((outgoing_tx, incoming_rx, stop_ipc_tx))
}
