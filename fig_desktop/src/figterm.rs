use std::collections::LinkedList;
use std::fmt::Display;
use std::ops::Deref;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use fig_proto::fig::EnvironmentVariable;
use fig_proto::local::{
    ShellContext,
    TerminalCursorCoordinates,
};
use fig_proto::secure::{
    hostbound,
    Clientbound,
};
use hashbrown::HashMap;
use parking_lot::lock_api::MutexGuard;
use parking_lot::{
    FairMutex,
    RawFairMutex,
};
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
    /// Linked list of `[FigtermSession]`s.
    pub linked_sessions: FairMutex<LinkedList<FigtermSession>>,
}

impl FigtermState {
    /// Inserts a new session id
    pub fn insert(&self, session: FigtermSession) {
        self.linked_sessions.lock().push_front(session);
    }

    /// Removes the given session id with a given lock
    pub fn remove_with_lock(
        &self,
        key: FigtermSessionId,
        guard: &mut MutexGuard<'_, RawFairMutex, LinkedList<FigtermSession>>,
    ) -> Option<FigtermSession> {
        self.remove_where_with_lock(|session| session.id == key, guard)
    }

    /// Removes the given session id with a given lock and closure
    pub fn remove_where_with_lock(
        &self,
        mut f: impl FnMut(&FigtermSession) -> bool,
        guard: &mut MutexGuard<'_, RawFairMutex, LinkedList<FigtermSession>>,
    ) -> Option<FigtermSession> {
        let mut sessions_temp = LinkedList::new();
        std::mem::swap(&mut **guard, &mut sessions_temp);
        let mut existing = None;
        guard.extend(sessions_temp.into_iter().filter_map(|x| {
            if f(&x) {
                existing = Some(x);
                None
            } else {
                Some(x)
            }
        }));
        existing
    }

    /// Gets mutable reference to the given session id and sets the most recent session id
    pub fn with_update<T>(&self, key: FigtermSessionId, f: impl FnOnce(&mut FigtermSession) -> T) -> Option<T> {
        let mut guard = self.linked_sessions.lock();

        self.remove_with_lock(key, &mut guard).map(|mut session| {
            let result = f(&mut session);
            guard.push_front(session);
            result
        })
    }

    pub fn with_most_recent<T>(&self, f: impl FnOnce(&mut FigtermSession) -> T) -> Option<T> {
        let mut guard = self.linked_sessions.lock();
        guard.iter_mut().find(|session| session.dead_since.is_none()).map(f)
    }

    pub fn with<T>(&self, session_id: &FigtermSessionId, f: impl FnOnce(&mut FigtermSession) -> T) -> Option<T> {
        let mut guard = self.linked_sessions.lock();
        guard.iter_mut().find(|session| &session.id == session_id).map(f)
    }

    pub fn with_maybe_id<T>(
        &self,
        session_id: &Option<FigtermSessionId>,
        f: impl FnOnce(&mut FigtermSession) -> T,
    ) -> Option<T> {
        match session_id {
            Some(session_id) => self.with(session_id, f),
            None => self.with_most_recent(f),
        }
    }
}

#[derive(Debug)]
pub struct FigtermSession {
    pub id: FigtermSessionId,
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

#[derive(Debug)]
pub struct FigtermSessionInfo {
    pub edit_buffer: EditBuffer,
    pub context: Option<ShellContext>,
}

impl FigtermSession {
    #[allow(dead_code)]
    pub fn get_info(&self) -> FigtermSessionInfo {
        FigtermSessionInfo {
            edit_buffer: self.edit_buffer.clone(),
            context: self.context.clone(),
        }
    }
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

pub async fn clean_figterm_cache(state: Arc<FigtermState>) {
    loop {
        trace!("cleaning figterm cache");
        let mut last_receive = Instant::now();
        {
            let mut guard = state.linked_sessions.lock();
            state.remove_where_with_lock(
                |session| {
                    if session.last_receive.elapsed() > Duration::from_secs(600) {
                        return true;
                    } else if last_receive > session.last_receive {
                        last_receive = session.last_receive;
                    }
                    false
                },
                &mut guard,
            );
        }
        sleep_until(last_receive + Duration::from_secs(600)).await;
    }
}
