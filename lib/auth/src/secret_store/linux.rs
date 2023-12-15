use super::Secret;
use crate::{
    Error,
    Result,
};

pub struct SecretStoreImpl {
    _private: (),
}

impl SecretStoreImpl {
    pub async fn new() -> Result<Self> {
        Ok(Self { _private: () })
    }

    /// Sets the `key` to `password` on the keychain, this will override any existing value
    pub async fn set(key: &str, password: &str) -> Result<()> {
        todo!()
    }

    /// Returns the password for the `key`
    ///
    /// If not found the result will be `Ok(None)`, other errors will be returned
    pub async fn get(key: &str) -> Result<Option<Secret>> {
        todo!()
    }

    /// Deletes the `key` from the keychain
    pub async fn delete(key: &str) -> Result<()> {
        todo!()
    }
}
