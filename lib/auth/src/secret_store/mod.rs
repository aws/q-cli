#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos::SecretStoreImpl;

use crate::Result;

pub struct SecretStore {
    secret_store_impl: SecretStoreImpl,
}

impl SecretStore {
    pub fn load() -> Result<Self> {
        SecretStoreImpl::load().map(|secret_store_impl| Self { secret_store_impl })
    }

    pub fn set(&self, key: &str, password: &str) -> Result<()> {
        self.secret_store_impl.set(key, password)
    }

    pub fn get(&self, key: &str) -> Result<Option<String>> {
        self.secret_store_impl.get(key)
    }

    pub fn delete(&self, key: &str) -> Result<()> {
        self.secret_store_impl.delete(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "not on ci"]
    fn test_set_password() {
        let store = SecretStore::load().unwrap();
        store.set("test", "test").unwrap();
        assert_eq!(store.get("test").unwrap().unwrap(), "test");
    }

    #[test]
    #[ignore = "not on ci"]
    fn secret_get_time() {
        let store = SecretStore::load().unwrap();
        store.set("test-key", "1234").unwrap();

        let now = std::time::Instant::now();
        for _ in 0..100 {
            store.get("test-key").unwrap();
        }

        println!("duration: {:?}", now.elapsed() / 100)
    }
}
