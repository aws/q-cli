use std::fmt;

use serde::{
    Deserialize,
    Serialize,
};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FigtermSessionId {
    Uuid(Uuid),
    String(String),
}

impl FigtermSessionId {
    pub fn new(s: impl AsRef<str> + Into<String>) -> Self {
        if let Ok(uuid) = Uuid::parse_str(s.as_ref()) {
            FigtermSessionId::Uuid(uuid)
        } else {
            FigtermSessionId::String(s.into())
        }
    }

    pub fn into_string(self) -> String {
        match self {
            FigtermSessionId::Uuid(uuid) => uuid.as_hyphenated().to_string(),
            FigtermSessionId::String(s) => s,
        }
    }
}

impl From<String> for FigtermSessionId {
    fn from(from: String) -> Self {
        FigtermSessionId::String(from)
    }
}

impl From<Uuid> for FigtermSessionId {
    fn from(from: Uuid) -> Self {
        FigtermSessionId::Uuid(from)
    }
}

impl fmt::Display for FigtermSessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FigtermSessionId::Uuid(uuid) => uuid.as_hyphenated().fmt(f),
            FigtermSessionId::String(s) => s.fmt(f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_uuid() {
        let uuid = uuid::Uuid::new_v4();
        let id = FigtermSessionId::new(uuid.as_hyphenated().to_string());
        assert_eq!(id, FigtermSessionId::Uuid(uuid));
    }

    #[test]
    fn test_new_string() {
        let id = FigtermSessionId::new("test");
        assert_eq!(id, FigtermSessionId::String("test".to_string()));
    }

    #[test]
    fn test_into_string() {
        let uuid = uuid::Uuid::new_v4();
        let id = FigtermSessionId::Uuid(uuid);
        assert_eq!(id.into_string(), uuid.as_hyphenated().to_string());
    }

    #[test]
    fn test_display() {
        let uuid = uuid::Uuid::new_v4();
        let id = FigtermSessionId::Uuid(uuid);
        assert_eq!(format!("{}", id), uuid.as_hyphenated().to_string());
    }
}
