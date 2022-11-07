use std::ops::Deref;

use serde::{
    Deserialize,
    Serialize,
};

/// A checksum for a plugin
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Checksum(String);

impl Checksum {
    pub fn new(value: impl Into<String>) -> Checksum {
        Checksum(value.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitChecksum(Checksum);

impl GitChecksum {
    pub fn new(value: impl Into<String>) -> GitChecksum {
        GitChecksum(Checksum::new(value))
    }
}

impl Deref for GitChecksum {
    type Target = Checksum;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
