use std::fmt::Display;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use fig_proto::figterm::{
    figterm_message,
    intercept_command,
    FigtermMessage,
    InsertTextCommand,
    InterceptCommand,
    SetBufferCommand,
};
use fig_proto::local::ShellContext;
use parking_lot::RwLock;
use tokio::sync::mpsc;
use tokio::time::{
    sleep_until,
    Instant,
};
use tracing::{
    error,
    trace,
};

use crate::GlobalState;

#[derive(Debug, Clone)]
pub struct FigTermSession {
    pub sender: mpsc::Sender<FigTermCommand>,
    pub last_receive: Instant,
    pub edit_buffer: EditBuffer,
    pub context: Option<ShellContext>,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct FigtermSessionId(pub String);

impl Deref for FigtermSessionId {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for FigtermSessionId {
    fn from(from: String) -> Self {
        FigtermSessionId(from)
    }
}

impl Display for FigtermSessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum FigTermCommand {
    SetInterceptAll,
    SetIntercept(Vec<char>),
    ClearIntercept,
    AddIntercept(Vec<char>),
    RemoveIntercept(Vec<char>),
    InsertText {
        insertion: Option<String>,
        deletion: Option<i64>,
        offset: Option<i64>,
        immediate: Option<bool>,
        insertion_buffer: Option<String>,
    },
    SetBuffer {
        text: String,
        cursor_position: Option<u64>,
    },
}

#[derive(Debug, Default)]
pub struct FigtermState {
    /// The most recent `[FigtermSessionId]` to be used.
    pub most_recent: RwLock<Option<FigtermSessionId>>,
    /// The list of `[FigtermSession]`s.
    pub sessions: DashMap<FigtermSessionId, FigTermSession, fnv::FnvBuildHasher>,
}

impl FigtermState {
    /// Set the most recent session.
    fn set_most_recent_session(&self, session_id: impl Into<Option<FigtermSessionId>>) {
        let session_id = session_id.into();
        trace!("Most recent session set to {:?}", session_id);
        *self.most_recent.write() = session_id;
    }

    /// Inserts a new session id
    pub fn insert(&self, key: FigtermSessionId, value: FigTermSession) {
        self.set_most_recent_session(key.clone());
        self.sessions.insert(key, value);
    }

    /// Removes the given session id
    pub fn remove(&self, key: &FigtermSessionId) -> Option<(FigtermSessionId, FigTermSession)> {
        if self.most_recent.read().as_ref() == Some(key) {
            self.set_most_recent_session(None);
        }
        self.sessions.remove(key)
    }

    /// Checks if the given session id is valid.
    pub fn contains_key(&self, key: &FigtermSessionId) -> bool {
        self.sessions.contains_key(key)
    }

    /// Gets mutable reference to the given session id and sets the most recent session id
    pub fn with_mut(&self, key: FigtermSessionId, f: impl FnOnce(&mut FigTermSession)) -> bool {
        match self.sessions.get_mut(&key) {
            Some(mut session) => {
                self.set_most_recent_session(key);
                f(&mut *session);
                true
            },
            None => false,
        }
    }

    pub fn most_recent_session(&self) -> Option<FigTermSession> {
        self.most_recent
            .read()
            .as_ref()
            .and_then(|key| self.sessions.get(key))
            .map(|session| session.value().clone())
    }
}

#[derive(Clone, Default, Debug)]
pub struct EditBuffer {
    pub text: String,
    pub cursor: i64,
}

pub fn ensure_figterm(session_id: FigtermSessionId, state: Arc<GlobalState>) {
    if state.figterm_state.contains_key(&session_id) {
        return;
    }
    let (tx, mut rx) = mpsc::channel(0xff);
    state.figterm_state.insert(session_id.clone(), FigTermSession {
        sender: tx,
        last_receive: Instant::now(),
        edit_buffer: EditBuffer::default(),
        context: None,
    });
    tokio::spawn(async move {
        let socket = fig_ipc::figterm::get_figterm_socket_path(&*session_id);

        let mut stream = match fig_ipc::connect_timeout(socket.clone(), Duration::from_secs(1)).await {
            Ok(stream) => stream,
            Err(err) => {
                error!("Error connecting to figterm socket at {:?}: {:?}", socket, err);
                return;
            },
        };

        trace!("figterm session {} opened", session_id);

        while let Some(command) = rx.recv().await {
            use figterm_message::Command;
            let message = match command {
                FigTermCommand::SetInterceptAll => Command::InterceptCommand(InterceptCommand {
                    intercept_command: Some(intercept_command::InterceptCommand::SetInterceptAll(
                        intercept_command::SetInterceptAll {},
                    )),
                }),
                FigTermCommand::SetIntercept(chars) => Command::InterceptCommand(InterceptCommand {
                    intercept_command: Some(intercept_command::InterceptCommand::SetIntercept(
                        intercept_command::SetIntercept {
                            chars: chars.into_iter().map(|x| x as u32).collect::<Vec<u32>>(),
                        },
                    )),
                }),
                FigTermCommand::ClearIntercept => Command::InterceptCommand(InterceptCommand {
                    intercept_command: Some(intercept_command::InterceptCommand::ClearIntercept(
                        intercept_command::ClearIntercept {},
                    )),
                }),
                FigTermCommand::AddIntercept(chars) => Command::InterceptCommand(InterceptCommand {
                    intercept_command: Some(intercept_command::InterceptCommand::AddIntercept(
                        intercept_command::AddIntercept {
                            chars: chars.into_iter().map(|x| x as u32).collect::<Vec<u32>>(),
                        },
                    )),
                }),
                FigTermCommand::RemoveIntercept(chars) => Command::InterceptCommand(InterceptCommand {
                    intercept_command: Some(intercept_command::InterceptCommand::RemoveIntercept(
                        intercept_command::RemoveIntercept {
                            chars: chars.into_iter().map(|x| x as u32).collect::<Vec<u32>>(),
                        },
                    )),
                }),
                FigTermCommand::InsertText {
                    insertion,
                    deletion,
                    offset,
                    immediate,
                    insertion_buffer,
                } => Command::InsertTextCommand(InsertTextCommand {
                    insertion,
                    deletion: deletion.map(|x| x as u64),
                    offset,
                    immediate,
                    insertion_buffer,
                }),
                FigTermCommand::SetBuffer { text, cursor_position } => {
                    Command::SetBufferCommand(SetBufferCommand { text, cursor_position })
                },
            };

            if let Err(err) = fig_ipc::send_message(&mut stream, FigtermMessage { command: Some(message) }).await {
                error!("Failed sending message to figterm session {}: {:?}", session_id, err);
                break;
            }

            if !state
                .figterm_state
                .with_mut(session_id.clone(), |session| session.last_receive = Instant::now())
            {
                break;
            }
        }
        // remove from cache
        trace!("figterm session {} closed", session_id);
        state.figterm_state.remove(&session_id);
    });
}

pub async fn clean_figterm_cache(state: Arc<GlobalState>) {
    loop {
        trace!("cleaning figterm cache");
        let mut last_receive = Instant::now();
        {
            let mut to_remove = Vec::new();
            for session in state.figterm_state.sessions.iter() {
                if session.last_receive.elapsed() > Duration::from_secs(600) {
                    to_remove.push(session.key().clone());
                } else if last_receive > session.last_receive {
                    last_receive = session.last_receive;
                }
            }
            for session_id in to_remove {
                state.figterm_state.remove(&session_id);
            }
        }
        sleep_until(last_receive + Duration::from_secs(600)).await;
    }
}
