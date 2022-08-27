use std::fmt::Display;
use std::hash::BuildHasherDefault;
use std::ops::Deref;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use fig_proto::fig::EnvironmentVariable;
use fig_proto::local::{
    ShellContext,
    TerminalCursorCoordinates,
};
use fig_proto::secure::{
    hostbound,
    Clientbound,
};
use fnv::FnvHasher;
use hashbrown::HashMap;
use parking_lot::FairMutex;
use time::OffsetDateTime;
use tokio::sync::oneshot;
use tokio::time::{
    sleep_until,
    Duration,
    Instant,
};
use tracing::trace;

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

#[derive(Clone, Default, Debug)]
pub struct EditBuffer {
    pub text: String,
    pub cursor: i64,
}

#[derive(Debug, Clone)]
pub struct SessionMetrics {
    pub start_time: OffsetDateTime,
    pub end_time: OffsetDateTime,
    pub num_insertions: i64,
    pub num_popups: i64,
}

impl SessionMetrics {
    pub fn new(start: OffsetDateTime) -> Self {
        Self {
            start_time: start,
            end_time: start,
            num_insertions: 0,
            num_popups: 0,
        }
    }
}

#[derive(Debug, Default)]
pub struct FigtermState {
    /// The most recent `[FigtermSessionId]` to be used.
    most_recent: FairMutex<Option<FigtermSessionId>>,
    /// The list of `[FigtermSession]`s.
    pub sessions: DashMap<FigtermSessionId, FigtermSession, fnv::FnvBuildHasher>,
}

impl FigtermState {
    /// Set the most recent session.
    pub fn set_most_recent_session(&self, session_id: impl Into<Option<FigtermSessionId>>) {
        let session_id = session_id.into();
        trace!("Most recent session set to {session_id:?}");
        *self.most_recent.lock() = session_id;
    }

    /// Inserts a new session id
    pub fn insert(&self, key: FigtermSessionId, value: FigtermSession) {
        self.sessions.insert(key, value);
    }

    #[allow(dead_code)]
    /// Removes the given session id
    pub fn remove(&self, key: &FigtermSessionId) -> Option<(FigtermSessionId, FigtermSession)> {
        if self.most_recent.lock().as_ref() == Some(key) {
            self.set_most_recent_session(None);
        }
        self.sessions.remove(key)
    }

    /// Gets mutable reference to the given session id and sets the most recent session id
    pub fn with_mut<T>(&self, key: FigtermSessionId, f: impl FnOnce(&mut FigtermSession) -> T) -> Option<T> {
        self.sessions.get_mut(&key).map(|mut session| f(&mut session))
    }

    pub fn most_recent_session_id(&self) -> Option<FigtermSessionId> {
        self.most_recent.lock().as_ref().cloned()
    }

    pub fn most_recent_session(
        &self,
    ) -> Option<Ref<'_, FigtermSessionId, FigtermSession, BuildHasherDefault<FnvHasher>>> {
        let id = self.most_recent_session_id();
        id.as_ref().and_then(|id| self.sessions.get(id))
    }
}

#[derive(Debug)]
pub struct FigtermSession {
    pub secret: String,
    pub sender: flume::Sender<FigtermCommand>,
    pub writer: Option<flume::Sender<Clientbound>>,
    pub dead_since: Option<Instant>, // TODO(mia): prune old sessions
    pub edit_buffer: EditBuffer,
    pub last_receive: Instant,
    pub context: Option<ShellContext>,
    pub terminal_cursor_coordinates: Option<TerminalCursorCoordinates>,
    pub current_session_metrics: Option<SessionMetrics>,
    pub response_map: HashMap<u64, oneshot::Sender<hostbound::response::Response>>,
    pub nonce_counter: Arc<AtomicU64>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum FigtermCommand {
    InterceptDefault,
    InterceptClear,
    InterceptFigJs {
        intercept_bound_keystrokes: bool,
        intercept_global_keystrokes: bool,
        actions: Vec<fig_proto::figterm::Action>,
    },
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
    RunProcess {
        channel: oneshot::Sender<hostbound::response::Response>,
        executable: String,
        arguments: Vec<String>,
        working_directory: Option<String>,
        env: Vec<EnvironmentVariable>,
    },
    PseudoterminalExecute {
        channel: oneshot::Sender<hostbound::response::Response>,
        command: String,
        working_directory: Option<String>,
        background_job: Option<bool>,
        is_pipelined: Option<bool>,
        env: Vec<EnvironmentVariable>,
    },
}

macro_rules! field {
    ($fn_name:ident: $enum_name:ident, $($field_name: ident: $field_type: ty),*,) => {
        pub fn $fn_name($($field_name: $field_type),*) -> (Self, oneshot::Receiver<hostbound::response::Response>) {
            let (tx, rx) = oneshot::channel();
            (Self::$enum_name {channel: tx, $($field_name),*}, rx)
        }
    };
}

impl FigtermCommand {
    field!(
        run_process: RunProcess,
        executable: String,
        arguments: Vec<String>,
        working_directory: Option<String>,
        env: Vec<EnvironmentVariable>,
    );

    field!(
        pseudoterminal_execute: PseudoterminalExecute,
        command: String,
        working_directory: Option<String>,
        background_job: Option<bool>,
        is_pipelined: Option<bool>,
        env: Vec<EnvironmentVariable>,
    );
}

#[allow(dead_code)]
pub async fn clean_figterm_cache(state: Arc<FigtermState>) {
    loop {
        trace!("cleaning figterm cache");
        let mut last_receive = Instant::now();
        {
            let mut to_remove = Vec::new();
            for session in state.sessions.iter() {
                if session.last_receive.elapsed() > Duration::from_secs(600) {
                    to_remove.push(session.key().clone());
                } else if last_receive > session.last_receive {
                    last_receive = session.last_receive;
                }
            }
            for session_id in to_remove {
                state.remove(&session_id);
            }
        }
        sleep_until(last_receive + Duration::from_secs(600)).await;
    }
}
