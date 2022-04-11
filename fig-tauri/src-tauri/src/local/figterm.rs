use std::time::Duration;

use fig_proto::figterm::{
    figterm_message, intercept_command, FigtermMessage, InsertTextCommand, InterceptCommand,
    SetBufferCommand,
};
use tokio::{
    sync::mpsc,
    time::{sleep_until, Instant},
};
use tracing::{error, trace};

use crate::state::STATE;

#[allow(unused)]
#[derive(Debug)]
pub enum FigTermCommand {
    SetIntercept(Vec<char>),
    ClearIntercept,
    AddIntercept(Vec<char>),
    RemoveIntercept(Vec<char>),
    InsertText {
        insertion: Option<String>,
        deletion: Option<u64>,
        offset: Option<i64>,
        immediate: Option<bool>,
    },
    SetBuffer {
        text: String,
        cursor_position: Option<u64>,
    },
}

#[derive(Debug)]
pub struct FigTermSession {
    pub sender: mpsc::Sender<FigTermCommand>,
    pub last_receive: Instant,
    pub edit_buffer: EditBuffer,
}

#[derive(Clone, Default, Debug)]
pub struct EditBuffer {
    pub text: String,
    pub cursor: i64,
}

pub fn ensure_figterm(session_id: String) {
    if STATE.figterm_sessions.contains_key(&session_id) {
        return;
    }
    let (tx, mut rx) = mpsc::channel(0xFF);
    STATE.figterm_sessions.insert(
        session_id.clone(),
        FigTermSession {
            sender: tx,
            last_receive: Instant::now(),
            edit_buffer: EditBuffer::default(),
        },
    );
    tokio::spawn(async move {
        let socket = fig_ipc::figterm::get_figterm_socket_path(&session_id);

        let mut stream =
            match fig_ipc::connect_timeout(socket.clone(), Duration::from_secs(1)).await {
                Ok(stream) => stream,
                Err(err) => {
                    error!(
                        "Error connecting to figterm socket at {:?}: {:?}",
                        socket, err
                    );
                    return;
                }
            };

        trace!("figterm session {} opened", session_id);

        while let Some(command) = rx.recv().await {
            use figterm_message::Command;
            let message = match command {
                FigTermCommand::SetIntercept(chars) => {
                    Command::InterceptCommand(InterceptCommand {
                        intercept_command: Some(intercept_command::InterceptCommand::SetIntercept(
                            intercept_command::SetIntercept {
                                chars: chars.into_iter().map(|x| x as u32).collect::<Vec<u32>>(),
                            },
                        )),
                    })
                }
                FigTermCommand::ClearIntercept => Command::InterceptCommand(InterceptCommand {
                    intercept_command: Some(intercept_command::InterceptCommand::ClearIntercept(
                        intercept_command::ClearIntercept {},
                    )),
                }),
                FigTermCommand::AddIntercept(chars) => {
                    Command::InterceptCommand(InterceptCommand {
                        intercept_command: Some(intercept_command::InterceptCommand::AddIntercept(
                            intercept_command::AddIntercept {
                                chars: chars.into_iter().map(|x| x as u32).collect::<Vec<u32>>(),
                            },
                        )),
                    })
                }
                FigTermCommand::RemoveIntercept(chars) => {
                    Command::InterceptCommand(InterceptCommand {
                        intercept_command: Some(
                            intercept_command::InterceptCommand::RemoveIntercept(
                                intercept_command::RemoveIntercept {
                                    chars: chars
                                        .into_iter()
                                        .map(|x| x as u32)
                                        .collect::<Vec<u32>>(),
                                },
                            ),
                        ),
                    })
                }
                FigTermCommand::InsertText {
                    insertion,
                    deletion,
                    offset,
                    immediate,
                } => Command::InsertTextCommand(InsertTextCommand {
                    insertion,
                    deletion,
                    offset,
                    immediate,
                }),
                FigTermCommand::SetBuffer {
                    text,
                    cursor_position,
                } => Command::SetBufferCommand(SetBufferCommand {
                    text,
                    cursor_position,
                }),
            };

            if let Err(err) = fig_ipc::send_message(
                &mut stream,
                FigtermMessage {
                    command: Some(message),
                },
            )
            .await
            {
                error!(
                    "Failed sending message to figterm session {}: {:?}",
                    session_id, err
                );
                break;
            }
            match STATE.figterm_sessions.get_mut(&session_id) {
                Some(mut session) => {
                    session.last_receive = Instant::now();
                }
                None => break,
            }
        }
        // remove from cache
        trace!("figterm session {} closed", session_id);
        STATE.figterm_sessions.remove(&session_id);
    });
}

pub async fn clean_figterm_cache() {
    loop {
        trace!("cleaning figterm cache");
        let mut last_receive = Instant::now();
        {
            let mut to_remove = Vec::new();
            for session in STATE.figterm_sessions.iter() {
                if session.last_receive.elapsed() > Duration::from_secs(600) {
                    to_remove.push(session.key().clone());
                } else if last_receive > session.last_receive {
                    last_receive = session.last_receive;
                }
            }
            for session_id in to_remove {
                STATE.figterm_sessions.remove(&session_id);
            }
        }
        sleep_until(last_receive + Duration::from_secs(600)).await;
    }
}
