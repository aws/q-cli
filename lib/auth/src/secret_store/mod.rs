#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "linux")]
use linux::SecretStoreImpl;
#[cfg(target_os = "macos")]
use macos::SecretStoreImpl;

use crate::Result;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct Secret(pub String);

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Secret").finish()
    }
}

impl<T> From<T> for Secret
where
    T: Into<String>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

pub struct SecretStore {
    _inner: SecretStoreImpl,
}

impl SecretStore {
    pub async fn new() -> Result<Self> {
        SecretStoreImpl::new().await.map(|_inner| Self { _inner })
    }

    pub async fn set(&self, key: &str, password: &str) -> Result<()> {
        SecretStoreImpl::set(key, password).await
    }

    pub async fn get(&self, key: &str) -> Result<Option<Secret>> {
        SecretStoreImpl::get(key).await
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        SecretStoreImpl::delete(key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "not on ci"]
    async fn test_set_password() {
        let key = "test_set_password";
        let store = SecretStore::new().await.unwrap();
        store.set(key, "test").await.unwrap();
        assert_eq!(store.get(key).await.unwrap().unwrap().0, "test");
        store.delete(key).await.unwrap();
    }

    #[tokio::test]
    #[ignore = "not on ci"]
    async fn secret_get_time() {
        let key = "test_secret_get_time";
        let store = SecretStore::new().await.unwrap();
        store.set(key, "1234").await.unwrap();

        let now = std::time::Instant::now();
        for _ in 0..100 {
            store.get(key).await.unwrap();
        }

        println!("duration: {:?}", now.elapsed() / 100);

        store.delete(key).await.unwrap();
    }

    #[tokio::test]
    #[ignore = "not on ci"]
    async fn secret_delete() {
        let key = "test_secret_delete";

        let store = SecretStore::new().await.unwrap();
        store.set(key, "1234").await.unwrap();
        assert_eq!(store.get(key).await.unwrap().unwrap().0, "1234");
        store.delete(key).await.unwrap();
        assert_eq!(store.get(key).await.unwrap(), None);
    }
}
