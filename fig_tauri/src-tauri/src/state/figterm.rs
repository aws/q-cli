use std::{fmt::Display, ops::Deref};

use dashmap::DashMap;
use parking_lot::RwLock;
use tracing::trace;

use crate::local::figterm::FigTermSession;

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
        write!(f, "{}", self.0)
    }
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
            }
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
