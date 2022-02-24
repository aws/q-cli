use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::ops::Deref;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sha256Checksum(Checksum);

impl Sha256Checksum {
    pub fn new(value: impl Into<String>) -> Sha256Checksum {
        Sha256Checksum(Checksum::new(value))
    }

    /// Compute the checksum of the given data
    pub fn compute(data: impl AsRef<[u8]>) -> Sha256Checksum {
        let mut hasher = sha2::Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize();
        Sha256Checksum(Checksum::new(format!("{:x}", hash)))
    }
}

impl Deref for Sha256Checksum {
    type Target = Checksum;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_checksum() {
        let data = "Hello, world!";
        let checksum = Sha256Checksum::compute(data);

        assert_eq!(
            checksum.as_str(),
            "315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3"
        );
    }

    #[test]
    fn test_seralize_deserialize() {
        let checksum = Sha256Checksum::compute("abcdef");
        let serialized = serde_json::to_string(&checksum).unwrap();
        let deserialized = serde_json::from_str(&serialized).unwrap();

        assert_eq!(checksum, deserialized);
    }
}
